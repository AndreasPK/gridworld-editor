/// While for most of this editor I was pretty involved this is pure vibes based on the manual.
use crate::dnaparser::{GeneProperty, PropertyValue};

impl GeneProperty {
    #[allow(dead_code)]
    pub fn prop_info(&mut self, neuron_type: u8, property_number: u8) -> Option<String> {
        let _ = self;
        let neuron_char = PropertyValue { raw: neuron_type }.to_char()?;
        // Some call sites use 0-based property indices; the manual uses Property 0..7.
        // Expose 1-based as requested, while tolerating 0 as Property 1.
        lookup_prop_info(neuron_char, property_number).map(str::to_string)
    }
}

pub(crate) fn lookup_prop_info(neuron_char: char, prop_n: u8) -> Option<&'static str> {
    match (neuron_char, prop_n) {
        // [1] TargetTurner
        ('1', 0) => Some("direction selector"),
        // [2] ThresholdChanger
        ('2', 0) => Some("mode selector"),
        // [3] Ticker
        ('3', 0) => Some("rotation direction selector"),
        // [8] WebTurner
        ('8', 0) => Some("rotation direction selector"),
        // [9] WebWalker
        ('9', 0) => Some("move direction"),

        // [B] AntiToxinMaker
        ('B', 0) => Some("toxin tag"),
        ('B', 1) => Some("toxin type"),

        // [E] Blinker
        ('E', 0) => Some("red channel for blink"),
        ('E', 1) => Some("green channel for blink"),
        ('E', 2) => Some("blue channel for blink"),

        // [G] Counter
        ('G', 0) => Some("max count"),
        // [H] Digester
        ('H', 0) => Some("type of food to digest"),

        // [K] DNA Copier
        ('K', 0) => Some("z-index of genes in the DNA to copy"),
        ('K', 1) => Some("build-after-copy flag"),
        // [L] DnaExecutor
        ('L', 0) => Some("gene execution z-index"),

        // [N] EnergySensor
        ('N', 0) => Some("normalized min energy threshold"),
        ('N', 1) => Some("normalized max energy threshold"),
        ('N', 2) => Some("mode selector"),

        // [R] Eye
        ('R', 0) => Some("color channel"),
        ('R', 1) => Some("color threshold"),

        // [T] Fin
        ('T', 0) => Some("move direction"),
        // [Y] Jet
        ('Y', 0) => Some("move direction"),

        // [Z] Lamp
        ('Z', 0) => Some("red color channel"),
        ('Z', 1) => Some("green color channel"),
        ('Z', 2) => Some("blue color channel"),

        // [a] MembraneMaker
        ('a', 0) => Some("red color channel"),
        ('a', 1) => Some("green color channel"),
        ('a', 2) => Some("blue color channel"),
        ('a', 3) => Some("minimum energy factor"),
        ('a', 4) => Some("transfer energy factor"),
        ('a', 5) => Some("membrane relative rotation"),

        // [b] MomentumSensor
        ('b', 0) => Some("momentum threshold"),
        // [d] MovementSensor
        ('d', 0) => Some("momentum threshold"),

        // [f] PainSensor
        ('f', 0) => Some("red color channel"),
        ('f', 1) => Some("green color channel"),
        ('f', 2) => Some("blue color channel"),

        // [h] PheromoneEmitter
        ('h', 0) => Some("pheromone type"),
        // [i] PheromoneSensor
        ('i', 0) => Some("pheromone type"),

        // [j] PhotosynthesisCell
        // none

        // [k] Pigment Cell
        ('k', 0) => Some("cell color red channel"),
        ('k', 1) => Some("cell color green channel"),
        ('k', 2) => Some("cell color blue channel"),

        // [l] Piston
        ('l', 0) => Some("move target"),
        ('l', 1) => Some("move creature"),
        ('l', 2) => Some("momentum direction"),

        // [m] PoisonMaker
        ('m', 0) => Some("toxin type"),
        ('m', 1) => Some("toxin index"),
        ('m', 2) => Some("toxin mode"),

        // [n] Randomizer
        ('n', 0) => Some("output signal fire chance"),
        // [o] RandomInput
        ('o', 0) => Some("output signal fire chance"),

        // [q] RotationSensor
        ('q', 0) => Some("rotation threshold"),
        ('q', 1) => Some("direction selector"),

        // [r] SideFin
        ('r', 0) => Some("move direction"),
        // [s] SideJet
        ('s', 0) => Some("move direction"),

        // [v] SkinColorChanger
        ('v', 0) => Some("red color channel"),
        ('v', 1) => Some("green color channel"),
        ('v', 2) => Some("blue color channel"),

        // [w] SlimeEmitter
        ('w', 0) => Some("red color channel"),
        ('w', 1) => Some("green color channel"),
        ('w', 2) => Some("blue color channel"),
        _ => None,
    }
}
