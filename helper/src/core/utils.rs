use rand::prelude::*;

pub struct Utils;

impl Utils {
    /// Generate a random token using UUIDv4.
    pub fn generate_token() -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
    }

    // 根据输入手机号的长度进行不同的脱敏
    // 11位手机号：138****1234
    // 10位手机号：138***1234
    // 7位手机号：13***1234
    // 其他长度：不脱敏
    pub fn mask_phone_number(phone: &str) -> String {
        let len = phone.len();
        match len {
            11 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(3..7, "****");
                masked_phone
            }
            10 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(3..6, "***");
                masked_phone
            }
            7 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(2..5, "***");
                masked_phone
            }
            _ => phone.to_string(), // 对于其他长度，不进行脱敏处理
        }
    }

    // 生成一个随机用户名
    // pub fn generate_username() -> String {
    //     let mut rng = rand::thread_rng();
    //     let username_length = rng.gen_range(6..=12);
    //     rng.sample_iter(&Alphanumeric)
    //         .take(username_length)
    //         .map(char::from)
    //         .collect()
    // }
    //

    /// 根据权重加权随机选择一个名称
    ///
    /// # 参数
    /// - `names`: 名称列表
    /// - `weights`: 权重列表（与名称一一对应）
    ///
    /// fn main() {
    //     let names = vec![
    //         "Alice".to_string(),
    //         "Bob".to_string(),
    //         "Charlie".to_string(),
    //     ];
    //     let weights = vec![1, 3, 6];

    //     if let Some(name) = utils::weighted_random::weighted_random_name(&names, &weights) {
    //         println!("加权随机选中的名称: {}", name);
    //     }
    // }
    /// # 返回
    /// - `Option<String>`: 随机选中的名称
    pub fn weighted_random_name(names: &[String], weights: &[usize]) -> Option<String> {
        if names.is_empty() || names.len() != weights.len() {
            return None;
        }
        let total: usize = weights.iter().sum();
        if total == 0 {
            return None;
        }
        let mut rng = rand::rng();
        let mut target = rng.random_range(0..total);
        for (name, &weight) in names.iter().zip(weights.iter()) {
            if target < weight {
                return Some(name.clone());
            }
            target -= weight;
        }
        None
    }

    /// 从名称列表中随机选取一个名称
    ///
    /// # 参数
    /// - `names`: 名称的切片引用
    ///
    /// # 返回
    /// - `Option<String>`: 随机选中的名称，如果列表为空则为 None
    pub fn random_name(names: &[String]) -> Option<String> {
        // 创建线程本地的随机数生成器
        let mut rng = rand::rng();
        // 随机选取一个元素并克隆为 String
        names.choose(&mut rng).cloned()
    }

    /// 尝试将字符串解析为 usize，如果失败则返回默认值
    //
    /// # 参数
    pub fn to_usize_or(s: &str, default: usize) -> usize {
        s.trim().parse::<usize>().unwrap_or(default)
    }
}
