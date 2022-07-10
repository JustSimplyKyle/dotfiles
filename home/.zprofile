export _JAVA_AWT_WM_NONREPARENTING=1
export QT_QPA_PLATFORMTHEME=qt5ct
if [ "$XDG_CURRENT_DESKTOP" = "hyprland" ] ; then
	export GTK_IM_MODULE DEFAULT=ibus
	export QT_IM_MODULE  DEFAULT=ibus
	export XMODIFIERS    DEFAULT=@im=ibus
	export INPUT_METHOD  DEFAULT=ibus
	export SDL_IM_MODULE DEFAULT=ibus
	export GLFW_IM_MODULE DEFAULT=ibus
fi
