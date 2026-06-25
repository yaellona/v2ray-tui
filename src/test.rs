use super::utils;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fetch_subscription(){
        match utils::fetch_subscription("https://103.14.76.98/sub/pianyi/dad9c33c10f77b1d892911351b527e7d") {
            Ok(nodes) => {
                nodes.save_to_config();
                println!("{:?}", nodes)
            },
            Err(e) => eprintln!("错误: {}", e),
        }
    }
}