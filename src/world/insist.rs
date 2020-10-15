use crate::math::vec::*;
use gamemath::Vec2;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_with::{DeserializeAs, SerializeAs};
use std::ops::{Add, AddAssign, Div, Mul, Neg};

/// A value and its velocity.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Insist<T> {
    pub state: T,
    pub velocity: T,
}

impl Insist<f32> {
    pub fn is_zero(self) -> bool {
        self.state == 0.0 && self.velocity == 0.0
    }
}

impl Insist<Vec2<f32>> {
    pub fn length_squared(&self) -> Insist<f32> {
        Insist {
            state: self.state.length_squared(),
            velocity: self.state.length_squared(),
        }
    }

    pub fn dot(&self, right: &Self) -> Insist<f32> {
        Insist {
            state: self.state.dot(right.state),
            velocity: self.velocity.dot(right.velocity),
        }
    }

    pub fn get_common(insists: Vec<&Insist<Vec2<f32>>>) -> Insist<Vec2<f32>> {
        let mut sum = Insist::default();
        for insist in &insists {
            sum += **insist;
        }

        let sum_norm = sum.length_squared();
        if sum_norm.state == 0.0 && sum_norm.velocity == 0.0 {
            return sum;
        }

        let mut projection_sum: Insist<f32> = Insist::default();
        for insist in &insists {
            projection_sum += sum.dot(insist) / sum_norm;
        }
        let projection_mean = projection_sum / insists.len() as f32;
        sum * projection_mean
    }
}

impl<T: AddAssign<T>> AddAssign<Insist<T>> for Insist<T> {
    fn add_assign(&mut self, insist: Insist<T>) {
        self.state += insist.state;
        self.velocity += insist.velocity;
    }
}

impl<T: AddAssign<T>> Add<Insist<T>> for Insist<T> {
    type Output = Insist<T>;

    fn add(self, right: Insist<T>) -> Insist<T> {
        let mut result = self;
        result += right;
        result
    }
}

impl<A: Mul<B, Output = O>, B, O> Mul<Insist<B>> for Insist<A> {
    type Output = Insist<O>;

    fn mul(self, insist: Insist<B>) -> Insist<O> {
        Insist {
            state: self.state * insist.state,
            velocity: self.velocity * insist.velocity,
        }
    }
}

impl<T: Div<T, Output = O>, O> Div<Insist<T>> for Insist<T> {
    type Output = Insist<O>;

    fn div(self, right: Insist<T>) -> Insist<O> {
        Insist {
            state: self.state / right.state,
            velocity: self.velocity / right.velocity,
        }
    }
}

impl<T: Div<T, Output = O> + Copy, O> Div<T> for Insist<T> {
    type Output = Insist<O>;

    fn div(self, right: T) -> Insist<O> {
        Insist {
            state: self.state / right,
            velocity: self.velocity / right,
        }
    }
}

impl<T: Neg<Output = O>, O> Neg for Insist<T> {
    type Output = Insist<O>;

    fn neg(self) -> Insist<O> {
        Insist {
            state: -self.state,
            velocity: -self.velocity,
        }
    }
}

impl<T: Serialize + Clone> SerializeAs<Insist<Vec2<T>>> for Insist<Vec2Serde<T>> {
    fn serialize_as<S>(source: &Insist<Vec2<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Insist", 3)?;
        s.serialize_field("state", &Vec2Serde::from(&source.state))?;
        s.serialize_field("velocity", &Vec2Serde::from(&source.velocity))?;
        s.end()
    }
}

impl<'de, T: Serialize + Deserialize<'de>> DeserializeAs<'de, Insist<Vec2<T>>>
    for Insist<Vec2Serde<T>>
{
    fn deserialize_as<D>(deserializer: D) -> Result<Insist<Vec2<T>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            State,
            Velocity,
        }
        struct InsistVisitor<U> {
            p: std::marker::PhantomData<U>,
        };

        impl<'de, T: Serialize + Deserialize<'de>> Visitor<'de> for InsistVisitor<T> {
            type Value = Insist<Vec2Serde<T>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Insist")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Insist<Vec2Serde<T>>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let state = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let velocity = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Insist { state, velocity })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Insist<Vec2Serde<T>>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut state = None;
                let mut velocity = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::State => {
                            if state.is_some() {
                                return Err(de::Error::duplicate_field("state"));
                            }
                            state = Some(map.next_value()?);
                        }
                        Field::Velocity => {
                            if velocity.is_some() {
                                return Err(de::Error::duplicate_field("velocity"));
                            }
                            velocity = Some(map.next_value()?);
                        }
                    }
                }
                let state = state.ok_or_else(|| de::Error::missing_field("state"))?;
                let velocity = velocity.ok_or_else(|| de::Error::missing_field("velocity"))?;
                Ok(Insist { state, velocity })
            }
        }

        const FIELDS: &[&str] = &["state", "velocity"];
        deserializer
            .deserialize_struct(
                "Insist",
                FIELDS,
                InsistVisitor::<T> {
                    p: std::marker::PhantomData,
                },
            )
            .map(|Insist { state, velocity }| Insist {
                state: state.into(),
                velocity: velocity.into(),
            })
    }
}
