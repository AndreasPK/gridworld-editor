#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreatureDNA {
    pub metadata: DnaMetadata,
    pub creature: CreatureData,
    pub cells: Vec<NeuronProperties>,
    pub dna: Vec<DnaData>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DnaMetadata {
    pub name: Option<String>,
    pub date: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreatureData {
    pub skin_color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridIndex2 {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridIndex3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeuronProperties {
    pub index: GridIndex2,
    pub encoded: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DnaData {
    pub dna_comment_name: Option<String>,
    pub dna_name: Option<DnaNameRecord>,
    pub dna_location: Option<GridIndex2>,
    pub dna_creator: Option<DnaCreatorRecord>,
    pub genes: Vec<GeneRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaNameRecord {
    pub index: GridIndex2,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaCreatorRecord {
    pub index: GridIndex2,
    pub creator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneRecord {
    pub index: GridIndex3,
    pub encoded: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedGeneInfo {
    pub neuron_type: NeuronType,
    pub tag: GeneTag,
    pub properties: [Option<GeneProperty>; 8],
    pub output_tags: Vec<OutputTag>,
    pub bias: GeneBias,
    pub mirroring: GeneMirroring,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NeuronType(pub PropertyValueType);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneTag(pub PropertyValueType);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneProperty(pub PropertyValueType);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputTag {
    pub tag: PropertyValueType,
    pub weight: PropertyValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneBias(pub PropertyValueType);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneMirroring(pub PropertyValueType);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyValue {
    pub raw: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyTag {
    PTNeuron, // *
    PTTag, // $

    // # @ % ^ + | { } Properties 1-8
    PTProp1,
    PTProp2,
    PTProp3,
    PTProp4,
    PTProp5,
    PTProp6,
    PTProp7,
    PTProp8,
    PTAmpersand, // &
    PTOutputTag, // [
    PTBias, // ~
    PTMirror, // _
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyValueType {
    #[default]
    PInt,
    PFloat,
    PThreshold,
    PWeight,
    PBias,
    PMirror,
}

impl PropertyValue {
    const VALUE_MAP: &'static [u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789?!";

    pub fn from_char(c: char) -> Option<Self> {
        if !c.is_ascii() {
            return None;
        }

        let raw = Self::VALUE_MAP
            .iter()
            .position(|mapped| *mapped == c as u8)
            .map(|idx| idx as u8)?;

        Some(Self { raw })
    }

    pub fn to_char(self) -> Option<char> {
        let idx = usize::from(self.raw);
        (idx < Self::VALUE_MAP.len()).then(|| Self::VALUE_MAP[idx] as char)
    }

    pub fn as_int(self) -> u8 {
        self.raw
    }
    pub fn as_float(self) -> f32 {
        // 0 .. 1
        self.raw as f32 / 63.0
    }
    pub fn as_threshold(self) -> f32 {
        // 0 .. 2.5
        self.raw as f32 * 2.5 / (63.0)
    }
    pub fn as_weight(self) -> f32 {
        // -2.5 .. 2.5
        self.raw as f32 * 5.0 / (63.0) - 2.5
    }
    pub fn as_bias(self) -> f32 {
        // 0 .. 2.5
        self.as_threshold()
    }

    const MIRROR_MAP: [&str; 15] = [
        "P", "P+X", "P+Y", "P+XY", "P+X+Y", "P+X+XY", "P+Y+XY", "P+X+Y+XY", "X", "Y", "XY", "X+Y",
        "X+XY", "Y+XY", "X+Y+XY",
    ];
    pub fn as_mirror(self) -> &'static str {
        Self::MIRROR_MAP[(self.raw % 16) as usize]
    }
}

pub mod parser {
    use nom::{
        IResult, Parser,
        character::complete::{anychar, char},
        combinator::map_opt,
        sequence::preceded,
    };

    use crate::dnaparser::{PropertyValue, PropertyTag, PropertyValueType};

    fn prop_value(input: &str) -> IResult<&str, PropertyValue> {
        map_opt(anychar, PropertyValue::from_char).parse(input)
    }



    #[test]
    fn test_prop_value() {
        let input = "*A";
        let parsed = prop_value(input);
        assert!(parsed.is_ok());
        let (rest, value) = parsed.unwrap();
        assert_eq!(rest, "");
        assert_eq!(value.raw, 0);
    }

    fn decode_gene_info(input: &str) -> IResult<&str, super::DecodedGeneInfo> {
        //
        /*TODO: Implement:
        Use the comments on PropertyTag to map characters to what key they are
        The goal is to decode gene info from a string like "*Y$m#7@0%a^9+3|M{U}M~m&W[vm[gW[cf[b4[vT[Dk[?m[S8[!n[rW[tN[fv[Bu[VQ[4T[wF[Xe[D7[VB[uX[?3[!l"

        Walk over the string. The string consists of a sequence of key-value pairs.
        For each pair the first character tells us what the value represents.
        After the key there is one character that represents the value, except for output tags which have two values, a tag and weight in that order.

        Write a nom parser that parses/decodes strings like this and allocates a DecodedGeneInfo based on it.
        */

    }

// pub struct DecodedGeneInfo {
//     pub neuron_type: NeuronType,
//     pub tag: GeneTag,
//     pub properties: [GeneProperty; 8],
//     pub output_tags: Vec<OutputTag>,
//     pub bias: GeneBias,
//     pub mirroring: GeneMirroring,
// }

}

