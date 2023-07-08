use std::{error::Error, io::Read};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

pub async fn deserialise_from_file_async<T: DeserializeOwned>(
    file_path: &str,
) -> Result<T, Box<dyn Error>> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let t = serde_yaml::from_str(&contents)?;
    Ok(t)
}

pub async fn write_to_async<T: Serialize>(file_path: &str, data: &T) -> Result<(), Box<dyn Error>> {
    let contents = serde_yaml::to_string(data)?;
    let mut file = File::create(file_path).await?;
    file.write_all(contents.as_bytes()).await?;
    Ok(())
}

pub fn deserialise_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, Box<dyn Error>> {
    let mut file = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let t = serde_yaml::from_str(&contents)?;
    Ok(t)
}

pub async fn file_exists_async(file_path: &str) -> bool {
    fs::metadata(file_path).await.is_ok()
}

pub fn file_exists(file_path: &str) -> bool {
    std::fs::metadata(file_path).is_ok()
}

pub trait Transpose<T> {
    fn transpose(self) -> Vec<Vec<T>>;
}
impl<T: Default> Transpose<T> for Vec<Vec<T>> {
    fn transpose(mut self) -> Vec<Vec<T>> {
        let max_len = self
            .iter()
            .fold(0, |size, vec| std::cmp::max(size, vec.len()));
        (0..max_len + 1)
            .filter_map(|i| {
                let result = (0..self.len())
                    .filter_map(|j| {
                        self.get_mut(j).and_then(|inner| {
                            let val = inner.get_mut(i)?;
                            Some(std::mem::take(val))
                        })
                    })
                    .collect::<Vec<T>>();
                if result.is_empty() {
                    None
                } else {
                    Some(result)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Transpose;
    #[test]
    fn transpose_tranposes_simple_matrix() {
        let input = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let result = input.transpose();
        let expected = vec![vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9]];
        assert_eq!(result, expected);
    }
    #[test]
    fn transpose_variable_length() {
        let matrix = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];
        let expected = vec![vec![1, 4, 6], vec![2, 5, 7], vec![3, 8], vec![9]];
        let result = matrix.transpose();
        assert_eq!(result, expected);
    }
}
