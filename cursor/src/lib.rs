use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use flate2::read::GzDecoder;
use prost::Message;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::io::Read;
use uuid::Uuid;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/cursor.rs"));
}

use proto::{ChatMessage, ResMessage};

#[derive(Debug)]
pub struct ChatInput {
    pub role: String,
    pub content: String,
}

fn process_chat_inputs(inputs: Vec<ChatInput>) -> (String, Vec<proto::chat_message::Message>) {
    // 收集 system 和 developer 指令
    let instructions = inputs
        .iter()
        .filter(|input| input.role == "system" || input.role == "developer")
        .map(|input| input.content.clone())
        .collect::<Vec<String>>()
        .join("\n\n");

    // 使用默认指令或收集到的指令
    let instructions = if instructions.is_empty() {
        "Respond in Chinese by default".to_string()
    } else {
        instructions
    };

    // 过滤出 user 和 assistant 对话
    let mut chat_inputs: Vec<ChatInput> = inputs
        .into_iter()
        .filter(|input| input.role == "user" || input.role == "assistant")
        .collect();

    // 处理空对话情况
    if chat_inputs.is_empty() {
        return (
            instructions,
            vec![proto::chat_message::Message {
                role: 1, // user
                content: " ".to_string(),
                message_id: Uuid::new_v4().to_string(),
            }],
        );
    }

    // 如果第一条是 assistant，插入空的 user 消息
    if chat_inputs
        .first()
        .map_or(false, |input| input.role == "assistant")
    {
        chat_inputs.insert(
            0,
            ChatInput {
                role: "user".to_string(),
                content: " ".to_string(),
            },
        );
    }

    // 处理连续相同角色的情况
    let mut i = 1;
    while i < chat_inputs.len() {
        if chat_inputs[i].role == chat_inputs[i - 1].role {
            let insert_role = if chat_inputs[i].role == "user" {
                "assistant"
            } else {
                "user"
            };
            chat_inputs.insert(
                i,
                ChatInput {
                    role: insert_role.to_string(),
                    content: " ".to_string(),
                },
            );
        }
        i += 1;
    }

    // 确保最后一条是 user
    if chat_inputs
        .last()
        .map_or(false, |input| input.role == "assistant")
    {
        chat_inputs.push(ChatInput {
            role: "user".to_string(),
            content: " ".to_string(),
        });
    }

    // 转换为 proto messages
    let messages = chat_inputs
        .into_iter()
        .map(|input| proto::chat_message::Message {
            role: if input.role == "user" { 1 } else { 2 },
            content: input.content,
            message_id: Uuid::new_v4().to_string(),
        })
        .collect();

    (instructions, messages)
}

pub async fn encode_chat_message(
    inputs: Vec<ChatInput>,
    model_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (instructions, messages) = process_chat_inputs(inputs);

    let chat = ChatMessage {
        messages,
        instructions: Some(proto::chat_message::Instructions {
            content: instructions,
        }),
        project_path: "/path/to/project".to_string(),
        model: Some(proto::chat_message::Model {
            name: model_name.to_string(),
            empty: String::new(),
        }),
        request_id: Uuid::new_v4().to_string(),
        summary: String::new(),
        conversation_id: Uuid::new_v4().to_string(),
    };

    let mut encoded = Vec::new();
    chat.encode(&mut encoded)?;

    let len_prefix = format!("{:010x}", encoded.len()).to_uppercase();
    let content = hex::encode_upper(&encoded);

    Ok(hex::decode(len_prefix + &content)?)
}

pub async fn decode_response(data: &[u8]) -> String {
    if let Ok(decoded) = decode_proto_messages(data) {
        if !decoded.is_empty() {
            return decoded;
        }
    }
    decompress_response(data).await
}

fn decode_proto_messages(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let hex_str = hex::encode(data);
    let mut pos = 0;
    let mut messages = Vec::new();

    while pos + 10 <= hex_str.len() {
        let msg_len = i64::from_str_radix(&hex_str[pos..pos + 10], 16)?;
        pos += 10;

        if pos + (msg_len * 2) as usize > hex_str.len() {
            break;
        }

        let msg_data = &hex_str[pos..pos + (msg_len * 2) as usize];
        pos += (msg_len * 2) as usize;

        let buffer = hex::decode(msg_data)?;
        let response = ResMessage::decode(&buffer[..])?;
        messages.push(response.msg);
    }

    Ok(messages.join(""))
}

async fn decompress_response(data: &[u8]) -> String {
    if data.len() <= 5 {
        return String::new();
    }

    let mut decoder = GzDecoder::new(&data[5..]);
    let mut text = String::new();

    match decoder.read_to_string(&mut text) {
        Ok(_) => {
            if !text.contains("<|BEGIN_SYSTEM|>") {
                text
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    }
}

pub fn generate_random_id(
    size: usize,
    dict_type: Option<&str>,
    custom_chars: Option<&str>,
) -> String {
    let charset = match (dict_type, custom_chars) {
        (_, Some(chars)) => chars,
        (Some("alphabet"), _) => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        (Some("max"), _) => "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-",
        _ => "0123456789",
    };

    let mut rng = thread_rng();
    (0..size)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect()
}

pub fn generate_hash() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    let mut hasher = Sha256::new();
    hasher.update(random_bytes);
    hex::encode(hasher.finalize())
}

fn obfuscate_bytes(bytes: &mut Vec<u8>) {
    let mut prev: u8 = 165;
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let old_value = *byte;
        *byte = (old_value ^ prev).wrapping_add((idx % 256) as u8);
        prev = *byte;
    }
}

pub fn generate_checksum(device_id: &str, mac_addr: Option<&str>) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 1_000_000;

    let mut timestamp_bytes = vec![
        ((timestamp >> 40) & 255) as u8,
        ((timestamp >> 32) & 255) as u8,
        ((timestamp >> 24) & 255) as u8,
        ((timestamp >> 16) & 255) as u8,
        ((timestamp >> 8) & 255) as u8,
        (255 & timestamp) as u8,
    ];

    obfuscate_bytes(&mut timestamp_bytes);
    let encoded = BASE64.encode(&timestamp_bytes);

    match mac_addr {
        Some(mac) => format!("{}{}/{}", encoded, device_id, mac),
        None => format!("{}{}", encoded, device_id),
    }
}
