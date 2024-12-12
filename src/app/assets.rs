pub struct Assets {
    pub settings_icon: egui::Image<'static>,
    pub reload_icon: egui::Image<'static>,
}

impl Assets {
    pub fn load() -> Self {
        let settings_icon = egui::Image::new(egui::include_image!("../../assets/settings.svg"))
            .tint(egui::Color32::BLACK);
        let reload_icon = egui::Image::new(egui::include_image!("../../assets/refresh.svg"))
            .tint(egui::Color32::BLACK);
        Assets {
            settings_icon,
            reload_icon,
        }
    }
}
