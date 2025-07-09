use cedar_policy::Entity;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Principal {
    entity: Entity,
    schema_id: String,
}

impl Principal {
    pub fn new(entity: Entity, schema_id: String) -> Self {
        Self { entity, schema_id }
    }

    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }

    pub fn get_schema_id(&self) -> &String {
        &self.schema_id
    }
}

impl Serialize for Principal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Principal", 2)?;
        state.serialize_field("entity", &self.entity.to_string())?;
        state.serialize_field("schema_id", &self.schema_id)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PrincipalData {
            entity: String,
            schema_id: String,
        }

        let data = PrincipalData::deserialize(deserializer)?;
        let entity = Entity::from_json_str(&data.entity, None).map_err(serde::de::Error::custom)?;
        Ok(Self::new(entity, data.schema_id))
    }
}
