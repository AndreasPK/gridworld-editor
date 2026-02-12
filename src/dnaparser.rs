#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::ops::{Deref, DerefMut};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreatureDNA {
    pub metadata: DnaMetadata,
    pub creature: CreatureData,
    pub cells: Cells,
    pub dna: Vec<DnaData>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cells(pub Vec<NeuronProperties>);

impl Cells {
    pub fn get_cell_at(&self, x: u16, y: u16) -> Option<&DecodedGeneInfo> {
        self.0
            .iter()
            .find(|cell| cell.index.x == x && cell.index.y == y)
            .map(|cell| &cell.decoded)
    }
}
impl Deref for Cells {
    type Target = Vec<NeuronProperties>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cells {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn parse_creature_dna(file_content: &str) -> Result<CreatureDNA> {
    CreatureDNA::parse(file_content)
}

impl CreatureDNA {
    pub fn parse(file_content: &str) -> Result<Self> {
        let mut out = Self::default();
        let mut current_dna: Option<usize> = None;

        for (line_no, raw_line) in file_content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(v) = line.strip_prefix("//name:") {
                out.metadata.name = Some(v.trim().to_string());
                continue;
            }
            if let Some(v) = line.strip_prefix("//date:") {
                out.metadata.date = Some(v.trim().to_string());
                continue;
            }
            if let Some(v) = line.strip_prefix("//version:") {
                out.metadata.version = Some(v.trim().to_string());
                continue;
            }
            if let Some(v) = line.strip_prefix("//dna:") {
                out.dna.push(DnaData {
                    dna_comment_name: Some(v.trim().to_string()),
                    ..DnaData::default()
                });
                current_dna = Some(out.dna.len() - 1);
                continue;
            }

            if let Some(v) = line.strip_prefix("skin_color = ") {
                out.creature.skin_color = Some(v.trim().to_string());
                continue;
            }
            if let Some((index, encoded)) = parse_index2_assignment(line, "neuron_properties") {
                let decoded = decode_gene_encoded(encoded, line_no + 1)?;
                out.cells.push(NeuronProperties { index, decoded });
                continue;
            }
            if let Some((index, name)) = parse_index2_assignment(line, "dna_name") {
                let i = ensure_current_dna(&mut out, &mut current_dna);
                out.dna[i].dna_name = Some(DnaNameRecord {
                    index,
                    name: name.to_string(),
                });
                continue;
            }
            if let Some((index, creator)) = parse_index2_assignment(line, "dna_creator") {
                let i = ensure_current_dna(&mut out, &mut current_dna);
                out.dna[i].dna_creator = Some(DnaCreatorRecord {
                    index,
                    creator: creator.to_string(),
                });
                continue;
            }
            if let Some(v) = line.strip_prefix("dna_location = ") {
                let location = parse_index2_bracket(v)
                    .ok_or_else(|| format!("invalid dna_location at line {}", line_no + 1))?;
                let i = ensure_current_dna(&mut out, &mut current_dna);
                out.dna[i].dna_location = Some(location);
                continue;
            }
            if let Some((index, encoded)) = parse_index3_assignment(line, "gene") {
                let decoded = decode_gene_encoded(encoded, line_no + 1)?;
                let i = ensure_current_dna(&mut out, &mut current_dna);
                out.dna[i].genes.push_gene(
                    index.z,
                    GeneRecord {
                        index: GridIndex2 {
                            x: index.x,
                            y: index.y,
                        },
                        decoded,
                    },
                );
                continue;
            }

            if line.starts_with("//") {
                continue;
            }

            return Err(format!("unrecognized line {}: {}", line_no + 1, line));
        }

        Ok(out)
    }

    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity(1024 + self.cells.len() * 96 + self.dna.len() * 256);

        out.push_str("/////////////////////////////////////////////////////////////////////////////////////\n");
        if let Some(name) = &self.metadata.name {
            let _ = writeln!(out, "//name:    {}", name);
        }
        if let Some(date) = &self.metadata.date {
            let _ = writeln!(out, "//date:    {}", date);
        }
        if let Some(version) = &self.metadata.version {
            let _ = writeln!(out, "//version: {}", version);
        }
        out.push('\n');

        out.push_str("//creature: \n");
        if let Some(skin_color) = &self.creature.skin_color {
            let _ = writeln!(out, "skin_color = {}", skin_color);
        }
        out.push('\n');

        out.push_str("//cells: \n");
        for cell in self.cells.iter() {
            let _ = writeln!(
                out,
                "neuron_properties[{}][{}] = {}",
                cell.index.x,
                cell.index.y,
                cell.decoded.encode()
            );
        }
        out.push('\n');

        for dna in &self.dna {
            out.push_str("//dna:");
            if let Some(name) = &dna.dna_comment_name {
                out.push(' ');
                out.push_str(name);
            }
            out.push('\n');

            if let Some(dna_name) = &dna.dna_name {
                let _ = writeln!(
                    out,
                    "dna_name[{}][{}] = {}",
                    dna_name.index.x, dna_name.index.y, dna_name.name
                );
            }
            if let Some(location) = dna.dna_location {
                let _ = writeln!(out, "dna_location = [{}][{}]", location.x, location.y);
            }
            if let Some(dna_creator) = &dna.dna_creator {
                let _ = writeln!(
                    out,
                    "dna_creator[{}][{}] = {}",
                    dna_creator.index.x, dna_creator.index.y, dna_creator.creator
                );
            }
            out.push('\n');

            for layer in dna.genes.iter() {
                for gene in &layer.genes {
                    let _ = writeln!(
                        out,
                        "gene[{}][{}][{}] = {}",
                        gene.index.x,
                        gene.index.y,
                        layer.z_level,
                        gene.decoded.encode()
                    );
                }
            }
            out.push('\n');
        }

        if !self.comments.is_empty() {
            for comment in &self.comments {
                out.push_str(comment);
                out.push('\n');
            }
        }

        out
    }
}

#[inline]
fn ensure_current_dna(dna: &mut CreatureDNA, current_dna: &mut Option<usize>) -> usize {
    if let Some(i) = *current_dna {
        i
    } else {
        dna.dna.push(DnaData::default());
        let i = dna.dna.len() - 1;
        *current_dna = Some(i);
        i
    }
}

#[inline]
fn parse_u16_prefix(input: &str) -> Option<(u16, &str)> {
    let mut end = 0usize;
    for b in input.as_bytes() {
        if b.is_ascii_digit() {
            end += 1;
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let value = input[..end].parse::<u16>().ok()?;
    Some((value, &input[end..]))
}

#[inline]
fn parse_index2_bracket(input: &str) -> Option<GridIndex2> {
    let mut rest = input.trim();
    rest = rest.strip_prefix('[')?;
    let (x, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("][")?;
    let (y, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix(']')?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(GridIndex2 { x, y })
}

#[inline]
fn parse_index2_assignment<'a>(line: &'a str, key: &str) -> Option<(GridIndex2, &'a str)> {
    let mut rest = line.strip_prefix(key)?;
    rest = rest.strip_prefix('[')?;
    let (x, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("][")?;
    let (y, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("] = ")?;
    Some((GridIndex2 { x, y }, rest.trim()))
}

#[inline]
fn parse_index3_assignment<'a>(line: &'a str, key: &str) -> Option<(GridIndex3, &'a str)> {
    let mut rest = line.strip_prefix(key)?;
    rest = rest.strip_prefix('[')?;
    let (x, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("][")?;
    let (y, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("][")?;
    let (z, r) = parse_u16_prefix(rest)?;
    rest = r.strip_prefix("] = ")?;
    Some((GridIndex3 { x, y, z }, rest.trim()))
}

#[inline]
fn decode_gene_encoded(encoded: &str, line_no: usize) -> Result<DecodedGeneInfo> {
    let parsed = parser::decode_gene_info(encoded)
        .map_err(|_| format!("invalid gene encoding at line {}", line_no))?;
    if !parsed.0.is_empty() {
        return Err(format!(
            "trailing data in gene encoding at line {}",
            line_no
        ));
    }
    Ok(parsed.1)
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DnaMetadata {
    pub name: Option<String>, //Just a comment
    pub date: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreatureData {
    pub skin_color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridIndex2 {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridIndex3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuronProperties {
    pub index: GridIndex2,
    pub decoded: DecodedGeneInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DnaData {
    pub dna_comment_name: Option<String>,
    pub dna_name: Option<DnaNameRecord>,
    pub dna_location: Option<GridIndex2>,
    pub dna_creator: Option<DnaCreatorRecord>,
    pub genes: DnaGenes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnaNameRecord {
    pub index: GridIndex2,
    pub name: String, //9 characters
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnaCreatorRecord {
    pub index: GridIndex2,
    pub creator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneRecord {
    pub index: GridIndex2,
    pub decoded: DecodedGeneInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnaLayer {
    pub z_level: u16,
    pub genes: Vec<GeneRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DnaGenes(pub Vec<DnaLayer>);

impl DnaGenes {
    pub fn push_gene(&mut self, z_level: u16, gene: GeneRecord) {
        if let Some(layer) = self.0.iter_mut().find(|layer| layer.z_level == z_level) {
            layer.genes.push(gene);
            return;
        }
        self.0.push(DnaLayer {
            z_level,
            genes: vec![gene],
        });
    }

    pub fn gene_count(&self) -> usize {
        self.0.iter().map(|layer| layer.genes.len()).sum()
    }

    pub fn get_layer_gene_mut(
        &mut self,
        layer_idx: usize,
        gene_idx: usize,
    ) -> Option<&mut GeneRecord> {
        self.0
            .get_mut(layer_idx)
            .and_then(|layer| layer.genes.get_mut(gene_idx))
    }
}

impl Deref for DnaGenes {
    type Target = Vec<DnaLayer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DnaGenes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Represents both neuron properties as well as gene data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DecodedGeneInfo {
    pub neuron_type: NeuronType,
    pub tag: GeneTag,
    pub properties: [GeneProperty; 8],
    pub bias: GeneBias,
    pub ampersand: Option<GeneProperty>,
    pub mirroring: GeneMirroring,
    pub output_tags: Vec<OutputTag>,
}

impl DecodedGeneInfo {
    pub fn encode(&self) -> String {
        let mut out = String::with_capacity(16 + self.output_tags.len() * 3);
        self.neuron_type.encode(&mut out);
        self.tag.encode(&mut out);
        for (idx, prop) in self.properties.iter().enumerate() {
            prop.encode(idx, &mut out);
        }
        self.bias.encode(&mut out);
        if let Some(ampersand) = self.ampersand {
            ampersand.encode_ampersand(&mut out);
        }
        if self.mirroring.0.raw != 0 {
            self.mirroring.encode(&mut out);
        }
        for output_tag in &self.output_tags {
            output_tag.encode(&mut out);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuronType(pub PropertyValue);

impl Default for NeuronType {
    fn default() -> Self {
        Self(PropertyValue::from_char('D').unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneTag(pub PropertyValue);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneProperty(pub PropertyValue);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OutputTag {
    pub tag: PropertyValue,
    pub weight: PropertyValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneBias(pub PropertyValue);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneMirroring(pub PropertyValue);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PropertyValue {
    pub raw: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyTag {
    PTNeuron, // *
    PTTag,    // $

    // # @ % ^ + | { } Properties 0-7
    PTProp0,
    PTProp1,
    PTProp2,
    PTProp3,
    PTProp4,
    PTProp5,
    PTProp6,
    PTProp7,
    PTBias,      // ~
    PTAmpersand, // &
    PTMirror,    // _
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
    const NAME_MAP: [&'static str; 64] = [
        "antenna",
        "anti-toxin emitter",
        "armor",
        "blank cell",
        "blinker",
        "cell color sensor",
        "counter",
        "digester cell",
        "direction sensor",
        "dna",
        "dna copier",
        "dna builder",
        "energy change sensor",
        "energy sensor",
        "energy sharer",
        "external receiver",
        "external sender",
        "eye",
        "feeder",
        "fin",
        "fuser",
        "group connection sensor",
        "group connector",
        "group disconnector",
        "jet",
        "lamp",
        "membrane maker",
        "momentum sensor",
        "mouth",
        "movement sensor",
        "neuron",
        "pain sensor",
        "painter",
        "pheromone emitter",
        "pheromone sensor",
        "photosynthesis",
        "pigment cell",
        "piston",
        "poison maker",
        "randomizer",
        "random input",
        "relative sensor",
        "rotation sensor",
        "side fin",
        "side jet",
        "signal receiver",
        "signal sender",
        "skin color changer",
        "slime emitter",
        "slippery cell",
        "sticky cell",
        "stinger",
        "storage cell",
        "target turner",
        "threshold changer",
        "ticker",
        "turner",
        "venom emitter",
        "web emitter",
        "web sensor",
        "web turner",
        "web walker",
        "unknown",
        "unknown",
    ];

    #[inline]
    pub fn to_char(self) -> String {
        GeneProperty(self.0).to_char()
    }

    #[inline]
    pub fn to_name(self) -> &'static str {
        let idx = usize::from(self.0.raw);
        Self::NAME_MAP.get(idx).copied().unwrap_or("invalid")
    }

    #[inline]
    fn encode(self, out: &mut String) {
        out.push('*');
        out.push(
            self.0
                .to_char()
                .expect("invalid PropertyValue for NeuronType"),
        );
    }
}

impl GeneTag {
    #[inline]
    pub fn to_char(self) -> String {
        GeneProperty(self.0).to_char()
    }

    #[inline]
    fn encode(self, out: &mut String) {
        out.push('$');
        out.push(self.0.to_char().expect("invalid PropertyValue for GeneTag"));
    }
}

impl GeneProperty {
    #[inline]
    pub fn to_char(self) -> String {
        match self.0.to_char() {
            Some(c) => c.to_string(),
            None => String::new(),
        }
    }

    #[inline]
    pub fn to_int(self) -> String {
        self.0.as_int().to_string()
    }

    #[inline]
    pub fn to_weight(self) -> String {
        self.0.as_weight().to_string()
    }

    #[inline]
    pub fn to_bias(self) -> String {
        self.0.as_bias().to_string()
    }

    #[inline]
    pub fn to_mirror(self) -> String {
        self.0.as_mirror().to_string()
    }

    #[inline]
    fn encode(self, index: usize, out: &mut String) {
        const PROPERTY_TAGS: [char; 8] = ['#', '@', '%', '^', '+', '|', '{', '}'];
        out.push(PROPERTY_TAGS[index]);
        out.push(
            self.0
                .to_char()
                .expect("invalid PropertyValue for GeneProperty"),
        );
    }

    #[inline]
    fn encode_ampersand(self, out: &mut String) {
        out.push('&');
        out.push(
            self.0
                .to_char()
                .expect("invalid PropertyValue for GeneProperty"),
        );
    }
}

impl GeneBias {
    #[inline]
    pub fn to_bias(self) -> String {
        GeneProperty(self.0).to_bias()
    }

    #[inline]
    fn encode(self, out: &mut String) {
        out.push('~');
        out.push(
            self.0
                .to_char()
                .expect("invalid PropertyValue for GeneBias"),
        );
    }
}

impl GeneMirroring {
    #[inline]
    pub fn to_mirror(self) -> String {
        GeneProperty(self.0).to_mirror()
    }

    #[inline]
    fn encode(self, out: &mut String) {
        out.push('_');
        out.push(
            self.0
                .to_char()
                .expect("invalid PropertyValue for GeneMirroring"),
        );
    }
}

impl OutputTag {
    #[inline]
    pub fn to_char(self) -> String {
        GeneProperty(self.tag).to_char()
    }

    #[inline]
    pub fn to_weight(self) -> String {
        GeneProperty(self.weight).to_weight()
    }

    #[inline]
    fn encode(self, out: &mut String) {
        out.push('[');
        out.push(
            self.tag
                .to_char()
                .expect("invalid PropertyValue for OutputTag.tag"),
        );
        out.push(
            self.weight
                .to_char()
                .expect("invalid PropertyValue for OutputTag.weight"),
        );
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
        Self::MIRROR_MAP[(usize::from(self.raw)) % Self::MIRROR_MAP.len()]
    }

    pub fn increase(&mut self) {
        if self.raw < 63 {
            self.raw += 1;
        }
    }
    pub fn decrease(&mut self) {
        if self.raw > 0 {
            self.raw -= 1;
        }
    }
}

pub mod parser {
    use nom::{
        IResult, Parser,
        character::complete::anychar,
        combinator::map_opt,
        error::{Error, ErrorKind},
    };

    use crate::dnaparser::{PropertyTag, PropertyValue};

    fn prop_value(input: &str) -> IResult<&str, PropertyValue> {
        map_opt(anychar, PropertyValue::from_char).parse(input)
    }

    #[inline]
    fn property_tag_from_key(c: char) -> Option<PropertyTag> {
        match c {
            '*' => Some(PropertyTag::PTNeuron),
            '$' => Some(PropertyTag::PTTag),
            '#' => Some(PropertyTag::PTProp0),
            '@' => Some(PropertyTag::PTProp1),
            '%' => Some(PropertyTag::PTProp2),
            '^' => Some(PropertyTag::PTProp3),
            '+' => Some(PropertyTag::PTProp4),
            '|' => Some(PropertyTag::PTProp5),
            '{' => Some(PropertyTag::PTProp6),
            '}' => Some(PropertyTag::PTProp7),
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

    // Can decode both neuron properties as well as gene data
    pub(crate) fn decode_gene_info(input: &str) -> IResult<&str, super::DecodedGeneInfo> {
        let mut info = super::DecodedGeneInfo {
            neuron_type: super::NeuronType(PropertyValue::default()),
            tag: super::GeneTag(PropertyValue::default()),
            properties: [super::GeneProperty(PropertyValue::default()); 8],
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
                | PropertyTag::PTProp0
                | PropertyTag::PTProp1
                | PropertyTag::PTProp2
                | PropertyTag::PTProp3
                | PropertyTag::PTProp4
                | PropertyTag::PTProp5
                | PropertyTag::PTProp6
                | PropertyTag::PTProp7
                | PropertyTag::PTAmpersand
                | PropertyTag::PTBias
                | PropertyTag::PTMirror => {
                    let (r, value) = prop_value(rest)?;
                    match tag {
                        PropertyTag::PTNeuron => info.neuron_type = super::NeuronType(value),
                        PropertyTag::PTTag => info.tag = super::GeneTag(value),
                        PropertyTag::PTProp0 => info.properties[0] = super::GeneProperty(value),
                        PropertyTag::PTProp1 => info.properties[1] = super::GeneProperty(value),
                        PropertyTag::PTProp2 => info.properties[2] = super::GeneProperty(value),
                        PropertyTag::PTProp3 => info.properties[3] = super::GeneProperty(value),
                        PropertyTag::PTProp4 => info.properties[4] = super::GeneProperty(value),
                        PropertyTag::PTProp5 => info.properties[5] = super::GeneProperty(value),
                        PropertyTag::PTProp6 => info.properties[6] = super::GeneProperty(value),
                        PropertyTag::PTProp7 => info.properties[7] = super::GeneProperty(value),
                        PropertyTag::PTBias => info.bias = super::GeneBias(value),
                        PropertyTag::PTAmpersand => {
                            info.ampersand = Some(super::GeneProperty(value))
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, time::SystemTime};

    #[test]
    fn parse_write_reparse_e5() {
        let input = fs::read_to_string("data/e5.txt").expect("failed to read data/e5.txt");
        let first = parse_creature_dna(&input).expect("failed to parse e5");
        let serialized = first.to_text();

        let mut path = std::env::temp_dir();
        path.push(temp_filename("creature_dna_roundtrip", "txt"));
        fs::write(&path, serialized).expect("failed to write temp dna file");

        let reparsed_input = fs::read_to_string(&path).expect("failed to read temp dna file");
        let second = parse_creature_dna(&reparsed_input).expect("failed to reparse temp dna file");
        let _ = fs::remove_file(&path);

        assert_eq!(first, second);
    }

    fn temp_filename(prefix: &str, ext: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let pid = std::process::id();
        PathBuf::from(format!("{prefix}_{pid}_{nanos}.{ext}"))
    }

    #[test]
    fn mirror_map_wraps_without_panic() {
        assert_eq!(PropertyValue { raw: 14 }.as_mirror(), "X+Y+XY");
        assert_eq!(PropertyValue { raw: 15 }.as_mirror(), "P");
    }
}
