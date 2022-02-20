use serde_json::{
    json,
    Value as JsonValue
};
use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::Debug
};

trait CustomCommand{
    fn command_type(&self) -> & 'static str;
    fn to_json(&self) -> JsonValue;
    fn from_json(json: &JsonValue) -> Self;
}

#[derive(Eq)]
enum Command<T>{
    SingleConfiguration{
        old_configuration: HashSet<usize>,
        configuration: HashSet<usize>
    },
    JointConfiguration{
        old_configuration: HashSet<usize>,
        new_configuration: HashSet<usize>    
    },
    Custom(T),
}

impl<T: CustomCommand> Command <T> {

    fn command_type(&self) -> &str {
        match self{ 
            Command::SingleConfiguration{..} => "SingleConfiguration",
            Command::JointConfiguration{..} =>  "JointConfiguration",
            Command::Custom(custom_command) => custom_command.command_type(),
    }
 
    fn to_json(&self) -> JsonValue{
        match self{ 
            Command::SingleConfiguration{configuration, old_configuration} => {
                        let mut configuration = configuration
                        .iter()
                        .copied()
                        .collect::<Vec<_>>();
            
                    configuration.sort_unstable();
            
                    let mut old_configuration = old_configuration
                        .iter()
                        .copied()
                        .collect::<Vec<_>>();
            
                    old_configuration.sort_unstable();
            
                    json!({
                        "configuration":{
                            "instanceIds": configuration
                        },
                        "oldConfiguration":{
                            "instanceIds": old_configuration
                                
                        },
            
                    })
            },
            Command::JointConfiguration{new_configuration, old_configuration} =>  {
                    let mut new_configuration = new_configuration
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
        
                configuration.sort_unstable();
        
                let mut old_configuration = old_configuration
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
        
                old_configuration.sort_unstable();
        
                json!({
                    "newConfiguration":{
                        "instanceIds": new_configuration
                    },
                    "oldConfiguration":{
                        "instanceIds": old_configuration
                            
                    },
        
                })
            },
            Command::Custom(custom_command) => custom_command.to_json(),
         }  


    }
}

impl <T: CustomCommand> TryFrom<&JsonValue> for Command<T>{
    type Error = ();
    fn try_from(json: &JsonValue) -> Result<Self, Self::Error> {
        json.get("type").and_then(JsonValue::as_str)
        .and_then(|command| 
        match command{
            "SingleConfiguration" => json.get("command").map(|command| {
                    Command::SingleConfiguration{
                        configuration: command
                            .get("configuration")
                            .map(decode_instance_ids)
                            .unwrap_or_else(HashSet::new),
                        old_configuration:command
                            .get("oldConfiguration")
                            .map(decode_instance_ids)
                            .unwrap_or_else(HashSet::new),
                        }
                    }),
                    "JointConfiguration" => json.get("command").map(|command| {
                        Command::JointConfiguration{
                            new_configuration: command
                                .get("newConfiguration")
                                .map(decode_instance_ids)
                                .unwrap_or_else(HashSet::new),
                            old_configuration:command
                                .get("oldConfiguration")
                                .map(decode_instance_ids)
                                .unwrap_or_else(HashSet::new),
                            }
                        })
                     _ => T::from_json(json),
                    })
                    .ok_or(())
    }
}

impl <T: Debug> Debug for Command <T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        match self {
            Self::SingleConfiguration{
                old_configuration,
                configuration,
            } =>{
                write!(&mut f, "SingleConfiguration({:?} -> {:?})", old_configuration, configuration)
            },
            Self::JointConfiguration {
                old_configuration
                new_configuration
            } => {
                write!(&mut f, "JointConfiguration({:?} -> {:?})", old_configuration, new_configuration)
            },
            Self::Custom(custom_comment) => custom_comment.fmt(f),
        }
    }
}

impl <T: PartialEq> PartialEq  for Command <T> {
    fn eq(&self, other: &self) -> bool{
        match self {
            Self::SingleConfiguration{
                old_configuration,
                configuration,
            } =>{
                if let Self::SingleConfiguration{
                    old_configuration: other_old_configuration,
                    configuration: other_configuration
                } = other {
                    old_configuration.eq(other_old_configuration)
                    && configuration.eq(other_configuration)
                } else {
                    false
                }
      
            },
            Self::JointConfiguration{
                old_configuration,
                new_configuration,
            } =>{
                if let Self::JointConfiguration{
                    old_configuration: other_old_configuration,
                    new_configuration: other_new_configuration
                } = other {
                    old_configuration.eq(other_old_configuration)
                    && new_configuration.eq(other_new_configuration)
                } else {
                    false
                }
      
            },
            Self::Custom(custom_comment) => custom_comment.eq(other),
        }
    }
    
}

#[derive(Debug, Eq, PartialEq)]
struct LogEntry <T>{
    term: usize,
    command: Option<Command<T>>,
}

impl <T: CustomCommand> LogEntry <T>{
    fn to_json(&self) -> JsonValue{
        let mut json = serde_json::Map::new();
        json.insert(String::from("term"), JsonValue::from(self.term));
        if let Some(command) = &self.command {
                json.insert(
                        String::from("type"),
                        JsonValue(command.command_type())
                );
                json.insert(String::from("command"), command.to_json();
        } 
        JsonValue::Object(json)
    }
}

fn decode_instance_ids(configuration: &JsonValue) -> HashSet<usize> {
    configuration
        .get("instaceIds")
        .and_then(JsonValue::as_array)
        .map(|instance_ids|{
            instance_ids
            .iter()
            .filter_map(JsonValue::as_u64)
            .map(|value| value as usize)
            .collect()
        })
        .unwrap_or_else(HashSet::new)

    }
}

impl From<&JsonValue> for LogEntry{
    fn from(json: &JsonValue) -> Self ({
        Self {
            term: json 
              .get("term")
              .and_then(JsonValue::as_u64)
              .map(|term| term as usize)
              .unwrap_or(0 ),
            command:  Command::try_from(json).ok(),
        }                     

    }
}

#[cfg(test)] 
mod tests{
    use super::*;
    use maplit::hashset;
    use serde_json::json;

    #[test]
    fn encode_single_configuration_command(){
        //Arrange
        let mut command = Command::SingleConfiguration{
            old_configuration: hashset!(5, 42, 85, 13531, 8354),
            configuration:  hashset!(42, 85, 13531, 8354),
        };

        let entry = LogEntry {
            term: 9,
            command: Some(command),
        };

        //Act
        assert_eq(
            json!({
                "type": "SingleConfiguration",
                "term": 9,
                "command": {
                    "configuration": {
                        "instanceIds":  [5, 42, 85, 8354, 13531],
                    },
                    "oldConfiguration": {
                        "instanceIds": [42, 85, 8354, 13531],
                    },
                }

            }),
            entry.to_json()
        );
    }

    #[test]
    fn decode_single_configuration_command(){
        //Arrange
        let encodedEntry = json!({
            "type": "SingleConfiguration",
            "term": 9,
            "command": {
                "oldConfiguration": {
                    "instanceIds": [5, 42, 85, 8354, 13531]
                },
                "configuration": {
                    "instanceIds": [42, 85, 8354, 13531]
                },
            }
        });
        //Act
        let LogEntry{
            term, 
            command
        } = LogEntry::from(&encoded_entry);
        assert_eq!(9, .term);
        assert!(command.is_some());
        let command = command.unwrap();
        assert_eq!("SingleConfiguration", command.command_type());
        match command {
            Command::SingleConfiguration{old_configuration, configuration} => {
                assert_eq!(
                     hashset!(42, 85, 13531, 8354),
                    configuration
                );
                assert_eq!(
                    hashset!(5, 42, 85, 13531, 8354),
                    old_configuration
                );
            },
            _ => panic!("expected `Command::SingleConfiguration`");
        }
    }
    #[test]
    fn encode_joint_configuration_command(){
        //Arrange
        let mut command = Command::JointConfiguration{
            old_configuration: hashset!(5, 42, 85, 13531, 8354),
            new_configuration:  hashset!(42, 85, 13531, 8354),
        };

        let entry = LogEntry {
            term: 9,
            command: Some(command),
        };

        //Act
        assert_eq(
            json!({
                "type": "JointConfiguration",
                "term": 9,
                "command": {
                    "configuration": {
                        "instanceIds":  [5, 42, 85, 8354, 13531],
                    },
                    "newConfiguration": {
                        "instanceIds": [42, 85, 8354, 13531],
                    },
                }

            }),
            entry.to_json()
        );
    }

    #[test]
    fn decode_joint_configuration_command(){
        //Arrange
        let encodedEntry = json!({
            "type": "JointConfiguration",
            "term": 9,
            "command": {
                "oldConfiguration": {
                    "instanceIds": [5, 42, 85, 8354, 13531]
                },
                "newConfiguration": {
                    "instanceIds": [42, 85, 8354, 13531]
                },
            }
        });

        //Act
        let LogEntry{
            term, 
            command
        } = LogEntry::from(encoded_entry);
        assert_eq!(9, .term);
        assert!(command.is_some());
        let command = command.unwrap();
        assert_eq!("JointConfiguration", command.command_type());
        match command {
            Command::JointConfiguration{old_configuration, new_configuration} => {
                assert_eq!(
                    hashset!(42, 85, 13531, 8354),
                    new_configuration
                );
                assert_eq!(
                    hashset!(5, 42, 85, 13531, 8354),
                    old_configuration
                );
            },
            _ => panic!("expected `Command::JointConfiguration`");
        }
    }

    #[test]
    fn to_json_without_command(){
        //Arrange
        let entry = LogEntry{ term: 9, command: None};

        //Act
        assert!(
            json!({
                {"term", 9}
            }),
            entry.to_json()
        );
    }

    #[test]
    fn from_json_without_command(){
        let entry_as_json = json!({
            "term": 9,
        });
        let entry = LogEntry::from(entry_as_json);
        assert_eq!(9, entry.term);
        assert!(entry.command == is_none());

         
    }

    #[test]
    fn compare_equal(){
        let examples = [
                    json!({
                        "type": "SingleConfiguration",
                        "term": 9,
                        "command": {
                            "oldConfiguration": {
                                "instanceIds": [5, 42, 85, 8354, 13531]
                            },
                            "configuration": {
                                "instanceIds": [42, 85, 8354, 13531]
                            },
                        }
                    }),
                    json!({
                        "type": "SingleConfiguration",
                        "term": 8,
                        "command": {
                            "oldConfiguration": {
                                "instanceIds": [5, 42, 85, 8354, 13531]
                            },
                            "configuration": {
                                "instanceIds": [42, 85, 8354, 13531]
                            },
                        }
                    }),
                    json!({
                        "type": "SingleConfiguration",
                        "term": 9,
                        "command": {
                            "oldConfiguration": {
                                "instanceIds": [5, 42, 85, 8354, 13531]
                            },
                            "configuration": {
                                "instanceIds": [5, 85, 8354, 13531]
                            },
                        }
                    }),
                    json!({
                        "term": 8,
                    }),
                    json!({
                        "term": 9,
                    })
                }
        ]
        .iter()
        .map(LogEntry::from)
        .collect::<Vec<_>>();
        let num_examples = examples.len();
        for i in 0..num_examples{
            for j in 0..num_examples{
                if i == j{
                    assert_eq(examples[i], examples[j]);
                }else{
                    assert_ne(examples[i], examples[j]);
                }
            }
        }

        #[test]
        fn custom_command(){
            struct PogChamp {
                payload: usize,
            }
    
            impl CustomCommand for PogChamp {
                fn command_type(&self) -> &'static str {
                    "PogChamp"
                }

                fn encode(&self) -> JsonValue{
                    json!({
                        "payload": self.payload,
                    })
                }
            }

            let pog_champ = PogChamp{
                payload: 42,
            };
            let pog_champ_entry;
            pog_champ_entry.term = 8;
            pog_champ_entry.command = pog_champ;
            const json::Value serializedpog_champ = pog_champ_entry;
            
            let pog_champ_factory = |command_as_json: JsonValue| {
                PogChamp{
                    payload: command_as_json.get("payload")
                        .map(JsonValue::as_u64)
                        .and_then(|payload| payload as usize)
                        .unwrap_or(0),
                }
            };
            let mut log_entry_factory = LogEntryFactory::new();
            log_entry_factory.register("PogChamp", pog_champ_factory);
        }
}