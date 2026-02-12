use egui::{Button, CollapsingHeader, Grid, Key, RichText, ScrollArea, Ui, WidgetText};

use crate::dnaparser::{
    CreatureDNA, DecodedGeneInfo, DnaCreatorRecord, DnaNameRecord, GeneRecord, GridIndex2,
    NeuronProperties, PropertyValue,
};
use crate::pdf_infos::lookup_prop_info;

mod grid_widget;

enum CellGridMode {
    Cells,
    DnaGenesLayer { dna_idx: usize, layer_idx: usize },
}

#[derive(Default)]
pub struct DnaWidget {
    selected_path: Option<String>,
}

impl DnaWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh_from_dna(&mut self) {
        self.selected_path = None;
    }

    pub fn sidebar_ui(&mut self, ui: &mut Ui, dna: Option<&CreatureDNA>) {
        ScrollArea::vertical().show(ui, |ui| {
            section(ui, "CreatureDNA", true, |ui| match dna {
                Some(dna) => self.loaded_tree_ui(ui, dna),
                None => self.empty_tree_ui(ui),
            });
        });
    }

    pub fn detail_ui(&mut self, ui: &mut Ui, mut dna: Option<&mut CreatureDNA>) {
        ui.label(RichText::new("Selected Component").strong());
        ui.separator();
        let Some(path) = self.selected_path.clone() else {
            ui.label("Select a tree item from the sidebar.");
            return;
        };

        ui.label(&path);
        ui.separator();

        if let Some(grid_mode) = cell_grid_mode_from_path(&path) {
            if let Some(dna_ref) = dna.as_deref_mut() {
                cell_grid_ui(ui, dna_ref, &mut self.selected_path, grid_mode);
            }
            ui.separator();
        }

        let detail_path = self.selected_path.clone().unwrap_or(path);
        ui.push_id(("dna_detail", &detail_path), |ui| {
            if let Some(dna_ref) = dna.as_deref_mut() {
                if selected_string_entry_ui(ui, dna_ref, &detail_path) {
                    return;
                }
            }

            match dna.and_then(|dna| selected_decoded_info_mut(dna, &detail_path)) {
                Some(gene_info) => decoded_info_ui(ui, gene_info),
                None => {
                    ui.label("No decoded details for the current selection.");
                }
            }
        });
    }

    fn empty_tree_ui(&mut self, ui: &mut Ui) {
        for section_name in ["metadata", "creature", "cells", "dna", "comments"] {
            self.leaf(ui, section_name, &format!("CreatureDNA/{section_name}"));
        }
    }

    fn loaded_tree_ui(&mut self, ui: &mut Ui, dna: &CreatureDNA) {
        self.metadata_ui(ui);
        self.creature_ui(ui);
        self.cells_ui(ui, dna);
        self.dna_ui(ui, dna);
        self.comments_ui(ui, dna);
    }

    fn metadata_ui(&mut self, ui: &mut Ui) {
        section(ui, "metadata", true, |ui| {
            self.leaf(ui, "name", "CreatureDNA/metadata/name");
            self.leaf(ui, "date", "CreatureDNA/metadata/date");
            self.leaf(ui, "version", "CreatureDNA/metadata/version");
        });
    }

    fn creature_ui(&mut self, ui: &mut Ui) {
        section(ui, "creature", true, |ui| {
            self.leaf(ui, "skin_color", "CreatureDNA/creature/skin_color");
        });
    }

    fn cells_ui(&mut self, ui: &mut Ui, dna: &CreatureDNA) {
        section(ui, format!("cells ({})", dna.cells.len()), true, |ui| {
            for (idx, cell) in dna.cells.iter().enumerate() {
                let neuron_label = format!(
                    "[{}][{}] {}",
                    cell.index.x,
                    cell.index.y,
                    cell.decoded.neuron_type.to_name()
                );
                self.leaf(ui, &neuron_label, &format!("CreatureDNA/cells/{idx}"));
            }
        });
    }

    fn dna_ui(&mut self, ui: &mut Ui, dna: &CreatureDNA) {
        section(ui, format!("dna ({})", dna.dna.len()), true, |ui| {
            for (dna_idx, dna_entry) in dna.dna.iter().enumerate() {
                let name = dna_entry
                    .dna_name
                    .as_ref()
                    .map(|n| n.name.as_str())
                    .or(dna_entry.dna_comment_name.as_deref())
                    .unwrap_or("unnamed");
                let block_label = format!("{} ({})", name, dna_entry.genes.gene_count());
                let block_path = format!("CreatureDNA/dna/{dna_idx}");
                section_with_id(ui, block_label, false, ("dna_block", dna_idx), |ui| {
                    self.leaf(
                        ui,
                        "dna_comment_name",
                        &format!("{block_path}/dna_comment_name"),
                    );
                    self.leaf(ui, "dna_name", &format!("{block_path}/dna_name"));
                    self.leaf(ui, "dna_location", &format!("{block_path}/dna_location"));
                    self.leaf(ui, "dna_creator", &format!("{block_path}/dna_creator"));

                    for (layer_idx, layer) in dna_entry.genes.iter().enumerate() {
                        section_with_id(
                            ui,
                            format!("Layer[{}]", layer.z_level),
                            false,
                            ("dna_layer", dna_idx, layer_idx),
                            |ui| {
                                for (gene_idx, gene) in layer.genes.iter().enumerate() {
                                    let gene_label = format!(
                                        "[{}][{}][{}] {}",
                                        gene.index.x,
                                        gene.index.y,
                                        layer.z_level,
                                        gene.decoded.neuron_type.to_name()
                                    );
                                    self.leaf(
                                        ui,
                                        &gene_label,
                                        &format!(
                                            "CreatureDNA/dna/{dna_idx}/genes/{layer_idx}/{gene_idx}"
                                        ),
                                    );
                                }
                            },
                        );
                    }
                });
            }
        });
    }

    fn comments_ui(&mut self, ui: &mut Ui, dna: &CreatureDNA) {
        section(
            ui,
            format!("comments ({})", dna.comments.len()),
            true,
            |ui| {
                for idx in 0..dna.comments.len() {
                    let label = format!("comment_{idx:03}");
                    self.leaf(ui, &label, &format!("CreatureDNA/comments/{label}"));
                }
            },
        );
    }

    fn leaf(&mut self, ui: &mut Ui, label: &str, path: &str) {
        let selected = self.selected_path.as_deref() == Some(path);
        if ui.selectable_label(selected, label).clicked() {
            self.selected_path = Some(path.to_owned());
        }
    }
}

fn cell_grid_ui(
    ui: &mut Ui,
    dna: &mut CreatureDNA,
    selected_path: &mut Option<String>,
    mode: CellGridMode,
) {
    let (max_x, max_y) = match mode {
        CellGridMode::Cells => {
            let max_x = dna
                .cells
                .iter()
                .map(|cell| cell.index.x)
                .max()
                .unwrap_or(8)
                .max(8);
            let max_y = dna
                .cells
                .iter()
                .map(|cell| cell.index.y)
                .max()
                .unwrap_or(8)
                .max(8);
            (max_x, max_y)
        }
        CellGridMode::DnaGenesLayer { dna_idx, layer_idx } => {
            let Some(dna_block) = dna.dna.get(dna_idx) else {
                ui.label("DNA block not found.");
                return;
            };
            let Some(layer) = dna_block.genes.get(layer_idx) else {
                ui.label("DNA layer not found.");
                return;
            };
            let max_x = layer
                .genes
                .iter()
                .map(|gene| gene.index.x)
                .max()
                .unwrap_or(8)
                .max(8);
            let max_y = layer
                .genes
                .iter()
                .map(|gene| gene.index.y)
                .max()
                .unwrap_or(8)
                .max(8);
            (max_x, max_y)
        }
    };

    let delete_pressed = ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace));
    if delete_pressed {
        match &mode {
            CellGridMode::Cells => {
                if let Some(selected_idx) = selected_path
                    .as_deref()
                    .and_then(selected_cell_idx_from_path)
                {
                    if selected_idx < dna.cells.len() {
                        dna.cells.remove(selected_idx);
                        *selected_path = Some("CreatureDNA/cells".to_owned());
                    }
                }
            }
            CellGridMode::DnaGenesLayer { dna_idx, layer_idx } => {
                if let Some((selected_dna_idx, selected_layer_idx, selected_gene_idx)) =
                    selected_path
                        .as_deref()
                        .and_then(selected_gene_idx_from_path)
                {
                    if selected_dna_idx == *dna_idx && selected_layer_idx == *layer_idx {
                        if let Some(layer) = dna
                            .dna
                            .get_mut(*dna_idx)
                            .and_then(|dna_block| dna_block.genes.get_mut(*layer_idx))
                        {
                            if selected_gene_idx < layer.genes.len() {
                                layer.genes.remove(selected_gene_idx);
                                if layer.genes.is_empty() {
                                    *selected_path = Some(format!("CreatureDNA/dna/{dna_idx}"));
                                } else {
                                    let next_idx = selected_gene_idx.min(layer.genes.len() - 1);
                                    *selected_path = Some(format!(
                                        "CreatureDNA/dna/{dna_idx}/genes/{layer_idx}/{next_idx}"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let columns = usize::from(max_x) + 1;
    let spacing_x = ui.spacing().item_spacing.x;
    let total_spacing = spacing_x * (columns.saturating_sub(1) as f32);
    let min_col_width = ((ui.available_width() - total_spacing) / columns as f32).max(14.0);

    let grid_id = match mode {
        CellGridMode::Cells => ("cells", 0, 0),
        CellGridMode::DnaGenesLayer { dna_idx, layer_idx } => ("dna", dna_idx, layer_idx),
    };
    Grid::new(("neurons", grid_id))
        .min_col_width(min_col_width)
        .show(ui, |ui| {
            for y in 0..=max_y {
                for x in 0..=max_x {
                    match mode {
                        CellGridMode::Cells => {
                            if let Some((idx, cell)) = dna
                                .cells
                                .iter()
                                .enumerate()
                                .find(|(_, cell)| cell.index.x == x && cell.index.y == y)
                            {
                                let cell_path = format!("CreatureDNA/cells/{idx}");
                                let selected = selected_path.as_deref() == Some(cell_path.as_str());
                                if grid_cell_response(
                                    ui,
                                    selected,
                                    cell.decoded.neuron_type.to_name(),
                                    min_col_width,
                                )
                                .clicked()
                                {
                                    *selected_path = Some(cell_path);
                                }
                            } else {
                                let response = grid_cell_response(ui, false, "_", min_col_width);
                                if response.double_clicked() {
                                    dna.cells.push(NeuronProperties {
                                        index: GridIndex2 { x, y },
                                        decoded: std::default::Default::default(),
                                    });
                                    let new_idx = dna.cells.len() - 1;
                                    *selected_path = Some(format!("CreatureDNA/cells/{new_idx}"));
                                }
                            }
                        }
                        CellGridMode::DnaGenesLayer { dna_idx, layer_idx } => {
                            let gene_at_pos = dna
                                .dna
                                .get(dna_idx)
                                .and_then(|dna_block| dna_block.genes.get(layer_idx))
                                .and_then(|layer| {
                                    layer
                                        .genes
                                        .iter()
                                        .enumerate()
                                        .find(|(_, gene)| gene.index.x == x && gene.index.y == y)
                                        .map(|(gene_idx, gene)| {
                                            (gene_idx, gene.decoded.neuron_type.to_name())
                                        })
                                });
                            if let Some((gene_idx, gene_name)) = gene_at_pos {
                                let gene_path = format!(
                                    "CreatureDNA/dna/{dna_idx}/genes/{layer_idx}/{gene_idx}"
                                );
                                let selected = selected_path.as_deref() == Some(gene_path.as_str());
                                if grid_cell_response(ui, selected, gene_name, min_col_width)
                                    .clicked()
                                {
                                    *selected_path = Some(gene_path);
                                }
                            } else {
                                let response = grid_cell_response(ui, false, "_", min_col_width);
                                if response.double_clicked() {
                                    if let Some(layer) = dna
                                        .dna
                                        .get_mut(dna_idx)
                                        .and_then(|dna_block| dna_block.genes.get_mut(layer_idx))
                                    {
                                        layer.genes.push(GeneRecord {
                                            index: GridIndex2 { x, y },
                                            decoded: std::default::Default::default(),
                                        });
                                        let new_idx = layer.genes.len() - 1;
                                        *selected_path = Some(format!(
                                            "CreatureDNA/dna/{dna_idx}/genes/{layer_idx}/{new_idx}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                ui.end_row();
            }
        });
}

fn selected_cell_idx_from_path(path: &str) -> Option<usize> {
    path.strip_prefix("CreatureDNA/cells/")?
        .parse::<usize>()
        .ok()
}

fn selected_gene_idx_from_path(path: &str) -> Option<(usize, usize, usize)> {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["CreatureDNA", "dna", dna_idx, "genes", layer_idx, gene_idx] => Some((
            dna_idx.parse::<usize>().ok()?,
            layer_idx.parse::<usize>().ok()?,
            gene_idx.parse::<usize>().ok()?,
        )),
        _ => None,
    }
}

fn cell_grid_mode_from_path(path: &str) -> Option<CellGridMode> {
    if path == "CreatureDNA/cells" || path.starts_with("CreatureDNA/cells/") {
        return Some(CellGridMode::Cells);
    }

    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["CreatureDNA", "dna", dna_idx, "genes", layer_idx, _gene_idx] => {
            Some(CellGridMode::DnaGenesLayer {
                dna_idx: dna_idx.parse::<usize>().ok()?,
                layer_idx: layer_idx.parse::<usize>().ok()?,
            })
        }
        _ => None,
    }
}

fn decoded_info_ui(ui: &mut Ui, gene_info: &mut DecodedGeneInfo) {
    let neuron_char = gene_info.neuron_type.0.to_char();

    ui.label(RichText::new("Decoded Info").strong());
    egui::Grid::new("decoded_info_grid")
        .striped(true)
        .num_columns(3)
        .show(ui, |ui| {
            ui.label("Neuron Type");
            let neuron_text = gene_info.neuron_type.to_name().to_string();
            property_value_ui(
                ui,
                "decoded_neuron_type",
                &mut gene_info.neuron_type.0,
                neuron_text,
            );
            ui.label("-");
            ui.end_row();

            ui.label("Tag");
            let tag_text = property_char(gene_info.tag.0);
            property_value_ui(ui, "decoded_tag", &mut gene_info.tag.0, tag_text);
            ui.label("-");
            ui.end_row();

            for (idx, prop) in gene_info.properties.iter_mut().enumerate() {
                ui.label(format!("Property {}", idx));
                let prop_text = property_char(prop.0);
                property_value_ui(
                    ui,
                    format!("decoded_property_{idx}"),
                    &mut prop.0,
                    prop_text,
                );
                let description = neuron_char
                    .and_then(|ch| lookup_prop_info(ch, (idx) as u8))
                    .unwrap_or("-");
                ui.label(description);
                ui.end_row();
            }

            ui.label("Bias");
            let bias_text = format!("{:.3}", gene_info.bias.0.as_bias());
            property_value_ui(ui, "decoded_bias", &mut gene_info.bias.0, bias_text);
            ui.label("-");
            ui.end_row();

            ui.label("Ampersand");
            if let Some(ampersand) = gene_info.ampersand.as_mut() {
                let ampersand_text = property_char(ampersand.0);
                property_value_ui(ui, "decoded_ampersand", &mut ampersand.0, ampersand_text);
            } else {
                ui.label("-");
            }
            ui.label("-");
            ui.end_row();

            ui.label("Mirroring");
            let mirror_text = gene_info.mirroring.0.as_mirror().to_string();
            property_value_ui(
                ui,
                "decoded_mirroring",
                &mut gene_info.mirroring.0,
                mirror_text,
            );
            ui.label("-");
            ui.end_row();
        });

    ui.separator();
    ui.label(RichText::new("Output Tags").strong());
    if gene_info.output_tags.is_empty() {
        ui.label("None");
        return;
    }

    egui::Grid::new("decoded_output_tags_grid")
        .striped(true)
        .num_columns(3)
        .show(ui, |ui| {
            for (idx, output_tag) in gene_info.output_tags.iter_mut().enumerate() {
                ui.label(format!("#{idx:02}"));
                let output_tag_text = property_char(output_tag.tag);
                property_value_ui(
                    ui,
                    format!("decoded_output_tag_{idx}"),
                    &mut output_tag.tag,
                    output_tag_text,
                );
                let output_weight_text = format!("{:.3}", output_tag.weight.as_weight());
                property_value_ui(
                    ui,
                    format!("decoded_output_weight_{idx}"),
                    &mut output_tag.weight,
                    output_weight_text,
                );
                ui.end_row();
            }
        });
}

fn string_edit_widget<F>(ui: &mut Ui, description: &str, value: &str, mut on_edit: F)
where
    F: FnMut(&str),
{
    ui.label(description);
    let mut edited = value.to_owned();
    let response = ui.text_edit_singleline(&mut edited);
    if response.changed() && edited != value {
        on_edit(&edited);
    }
}

fn selected_string_entry_ui(ui: &mut Ui, dna: &mut CreatureDNA, path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["CreatureDNA", "dna", block_idx, "dna_name"] => {
            let Some(block_idx) = block_idx.parse::<usize>().ok() else {
                return false;
            };
            let current = dna
                .dna
                .get(block_idx)
                .and_then(|block| block.dna_name.as_ref())
                .map(|record| record.name.clone())
                .unwrap_or_default();

            string_edit_widget(ui, "DNA Name", &current, |new_value| {
                if let Some(block) = dna.dna.get_mut(block_idx) {
                    if let Some(record) = block.dna_name.as_mut() {
                        record.name = new_value.to_owned();
                    } else {
                        block.dna_name = Some(DnaNameRecord {
                            index: block.dna_location.unwrap_or(GridIndex2 { x: 0, y: 0 }),
                            name: new_value.to_owned(),
                        });
                    }
                }
            });
            true
        }
        ["CreatureDNA", "dna", block_idx, "dna_creator"] => {
            let Some(block_idx) = block_idx.parse::<usize>().ok() else {
                return false;
            };
            let current = dna
                .dna
                .get(block_idx)
                .and_then(|block| block.dna_creator.as_ref())
                .map(|record| record.creator.clone())
                .unwrap_or_default();

            string_edit_widget(ui, "DNA Creator", &current, |new_value| {
                if let Some(block) = dna.dna.get_mut(block_idx) {
                    if let Some(record) = block.dna_creator.as_mut() {
                        record.creator = new_value.to_owned();
                    } else {
                        block.dna_creator = Some(DnaCreatorRecord {
                            index: block.dna_location.unwrap_or(GridIndex2 { x: 0, y: 0 }),
                            creator: new_value.to_owned(),
                        });
                    }
                }
            });
            true
        }
        _ => false,
    }
}

fn selected_decoded_info_mut<'a>(
    dna: &'a mut CreatureDNA,
    path: &str,
) -> Option<&'a mut DecodedGeneInfo> {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["CreatureDNA", "cells", cell_idx] => {
            let cell_idx = cell_idx.parse::<usize>().ok()?;
            dna.cells.get_mut(cell_idx).map(|cell| &mut cell.decoded)
        }
        [
            "CreatureDNA",
            "dna",
            block_idx,
            "genes",
            layer_idx,
            gene_idx,
        ] => {
            let block_idx = block_idx.parse::<usize>().ok()?;
            let layer_idx = layer_idx.parse::<usize>().ok()?;
            let gene_idx = gene_idx.parse::<usize>().ok()?;
            dna.dna
                .get_mut(block_idx)
                .and_then(|dna_block| dna_block.genes.get_layer_gene_mut(layer_idx, gene_idx))
                .map(|gene| &mut gene.decoded)
        }
        _ => None,
    }
}

fn property_value_ui<T: std::hash::Hash>(
    ui: &mut Ui,
    id_source: T,
    value: &mut PropertyValue,
    text: String,
) {
    let response = ui
        .push_id(id_source, |ui| {
            ui.add(egui::Label::new(text).sense(egui::Sense::hover()))
        })
        .inner;
    if response.hovered() {
        apply_scroll_step(ui, value);
    }
    response.on_hover_text("Scroll to increase/decrease");
}

fn apply_scroll_step(ui: &Ui, value: &mut PropertyValue) {
    let scroll_y = ui.ctx().input(|i| i.raw_scroll_delta.y);
    if scroll_y > 0.0 {
        value.increase();
    } else if scroll_y < 0.0 {
        value.decrease();
    }
}

fn property_char(value: PropertyValue) -> String {
    value.to_char().map(|c| c.to_string()).unwrap_or_default()
}

fn grid_cell_response(
    ui: &mut Ui,
    selected: bool,
    text: &str,
    min_col_width: f32,
) -> egui::Response {
    ui.add_sized(
        [min_col_width, ui.spacing().interact_size.y],
        Button::new(text).selected(selected),
    )
}

fn section<T, F>(ui: &mut Ui, title: T, default_open: bool, add_contents: F)
where
    T: Into<WidgetText>,
    F: FnOnce(&mut Ui),
{
    CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, add_contents);
}

fn section_with_id<T, I, F>(
    ui: &mut Ui,
    title: T,
    default_open: bool,
    id_source: I,
    add_contents: F,
) where
    T: Into<WidgetText>,
    I: std::hash::Hash,
    F: FnOnce(&mut Ui),
{
    CollapsingHeader::new(title)
        .id_salt(id_source)
        .default_open(default_open)
        .show(ui, add_contents);
}
