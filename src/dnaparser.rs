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
    pub bias: GeneBias,
    pub ampersand: Option<GeneProperty>,
    pub mirroring: GeneMirroring,
    pub output_tags: Vec<OutputTag>,
}

impl DecodedGeneInfo {
    pub fn encode(self) -> String {
        let mut out = String::with_capacity(16 + self.output_tags.len() * 3);
        self.neuron_type.encode(&mut out);
        self.tag.encode(&mut out);
        for (idx, prop) in self.properties.into_iter().enumerate() {
            if let Some(prop) = prop {
                prop.encode(idx, &mut out);
            }
        }
        self.bias.encode(&mut out);
        if let Some(ampersand) = self.ampersand {
            ampersand.encode_ampersand(&mut out);
        }
        if self.mirroring.0.raw != 0 {
            self.mirroring.encode(&mut out);
        }
        for output_tag in self.output_tags {
            output_tag.encode(&mut out);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NeuronType(pub PropertyValue);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneTag(pub PropertyValue);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneProperty(pub PropertyValue);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputTag {
    pub tag: PropertyValue,
    pub weight: PropertyValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneBias(pub PropertyValue);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneMirroring(pub PropertyValue);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    PTBias, // ~
    PTAmpersand, // &
    PTMirror, // _
    PTOutputTag, // [
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyValueRepresentation {
    #[default]
    PInt,
    PFloat,
    PThreshold,
    PWeight,
    PBias,
    PMirror,
}


impl NeuronType {
    #[inline]
    fn encode(self, out: &mut String) {
        out.push('*');
        out.push(self.0.to_char().expect("invalid PropertyValue for NeuronType"));
    }
}

impl GeneTag {
    #[inline]
    fn encode(self, out: &mut String) {
        out.push('$');
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneTag"));
    }
}

impl GeneProperty {
    #[inline]
    fn encode(self, index: usize, out: &mut String) {
        const PROPERTY_TAGS: [char; 8] = ['#', '@', '%', '^', '+', '|', '{', '}'];
        out.push(PROPERTY_TAGS[index]);
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneProperty"));
    }

    #[inline]
    fn encode_ampersand(self, out: &mut String) {
        out.push('&');
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneProperty"));
    }
}

impl GeneBias {
    #[inline]
    fn encode(self, out: &mut String) {
        out.push('~');
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneBias"));
    }
}

impl GeneMirroring {
    #[inline]
    fn encode(self, out: &mut String) {
        out.push('_');
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneMirroring"));
    }
}

impl OutputTag {
    #[inline]
    fn encode(self, out: &mut String) {
        out.push('[');
        out.push(self.tag.to_char().expect("invalid PropertyValue for OutputTag.tag"));
        out.push(self.weight.to_char().expect("invalid PropertyValue for OutputTag.weight"));
    }
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
        character::complete::anychar,
        combinator::map_opt,
        error::{Error, ErrorKind},
    };

    use crate::dnaparser::{PropertyValue, PropertyTag};

    fn prop_value(input: &str) -> IResult<&str, PropertyValue> {
        map_opt(anychar, PropertyValue::from_char).parse(input)
    }

    #[inline]
    fn property_tag_from_key(c: char) -> Option<PropertyTag> {
        match c {
            '*' => Some(PropertyTag::PTNeuron),
            '$' => Some(PropertyTag::PTTag),
            '#' => Some(PropertyTag::PTProp1),
            '@' => Some(PropertyTag::PTProp2),
            '%' => Some(PropertyTag::PTProp3),
            '^' => Some(PropertyTag::PTProp4),
            '+' => Some(PropertyTag::PTProp5),
            '|' => Some(PropertyTag::PTProp6),
            '{' => Some(PropertyTag::PTProp7),
            '}' => Some(PropertyTag::PTProp8),
            '~' => Some(PropertyTag::PTBias),
            '&' => Some(PropertyTag::PTAmpersand),
            '_' => Some(PropertyTag::PTMirror),
            '[' => Some(PropertyTag::PTOutputTag),
            _ => None,
        }
    }


    #[test]
    fn test_prop_value() {
        let input = "A";
        let parsed = prop_value(input);
        assert!(parsed.is_ok());
        let (rest, value) = parsed.unwrap();
        assert_eq!(rest, "");
        assert_eq!(value.raw, 0);
    }

    fn decode_gene_info(input: &str) -> IResult<&str, super::DecodedGeneInfo> {
        let mut info = super::DecodedGeneInfo {
            neuron_type: super::NeuronType(PropertyValue::default()),
            tag: super::GeneTag(PropertyValue::default()),
            properties: [None, None, None, None, None, None, None, None],
            output_tags: Vec::new(),
            bias: super::GeneBias(PropertyValue::default()),
            ampersand: None,
            mirroring: super::GeneMirroring(PropertyValue::default()),
        };

        let mut i = input;
        while !i.is_empty() {
            let mut chars = i.chars();
            let key = match chars.next() {
                Some(c) => c,
                None => break,
            };
            let key_len = key.len_utf8();
            let rest = &i[key_len..];

            let tag = match property_tag_from_key(key) {
                Some(t) => t,
                None => return Err(nom::Err::Error(Error::new(i, ErrorKind::Char))),
            };

            match tag {
                PropertyTag::PTOutputTag => {
                    let (r1, out_tag) = prop_value(rest)?;
                    let (r2, weight) = prop_value(r1)?;
                    info.output_tags.push(super::OutputTag {
                        tag: out_tag,
                        weight,
                    });
                    i = r2;
                }
                PropertyTag::PTNeuron
                | PropertyTag::PTTag
                | PropertyTag::PTProp1
                | PropertyTag::PTProp2
                | PropertyTag::PTProp3
                | PropertyTag::PTProp4
                | PropertyTag::PTProp5
                | PropertyTag::PTProp6
                | PropertyTag::PTProp7
                | PropertyTag::PTProp8
                | PropertyTag::PTAmpersand
                | PropertyTag::PTBias
                | PropertyTag::PTMirror => {
                    let (r, value) = prop_value(rest)?;
                    match tag {
                        PropertyTag::PTNeuron => info.neuron_type = super::NeuronType(value),
                        PropertyTag::PTTag => info.tag = super::GeneTag(value),
                        PropertyTag::PTProp1 => info.properties[0] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp2 => info.properties[1] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp3 => info.properties[2] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp4 => info.properties[3] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp5 => info.properties[4] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp6 => info.properties[5] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp7 => info.properties[6] = Some(super::GeneProperty(value)),
                        PropertyTag::PTProp8 => info.properties[7] = Some(super::GeneProperty(value)),
                        PropertyTag::PTBias => info.bias = super::GeneBias(value),
                        PropertyTag::PTAmpersand => info.ampersand = Some(super::GeneProperty(value)),
                        PropertyTag::PTMirror => info.mirroring = super::GeneMirroring(value),
                        PropertyTag::PTOutputTag => {}
                    }
                    i = r;
                }
            }
        }

        Ok((i, info))
    }

    #[test]
    fn test_gene_decoding() {
        let encoded = "*Y$m#7@0%a^9+3|M{U}M~m&W[vm[gW[cf[b4[vT[Dk[?m[S8[!n[rW[tN[fv[Bu[VQ[4T[wF[Xe[D7[VB[uX[?3[!l";
        let (rest, decoded) = decode_gene_info(encoded).unwrap();
        assert_eq!(rest, "");
        assert_eq!(decoded.encode(), encoded);

        let encoded = "*e$W#a@U%N^?+c|1{J}R~F&k_n[Ab[IP[S3[?g[mZ[19[KR[eI[3A[2t[ks[qs[Gv[r5[mn[n8[JM[SW[mP[Rz[QJ[WR";
        let (rest, decoded) = decode_gene_info(encoded).unwrap();
        assert_eq!(rest, "");
        assert_eq!(decoded.encode(), encoded);
    }


}
