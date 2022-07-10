local wezterm = require 'wezterm';

return {
    font = wezterm.font("JetBrains Mono", {weight="Bold",}),
    window_background_opacity= 0.89,
    font_size = 13.0,
    default_cursor_style = "BlinkingUnderline",
    animation_fps = 30,
    use_ime = true,
    enable_wayland=false,
    enable_csi_u_key_encoding = true,
}
