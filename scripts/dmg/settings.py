"""Finder layout for the signed H app; dmgbuild generates .DS_Store without AppleScript."""
from pathlib import Path
import plistlib

application = Path(defines['app'])  # supplied by dmgbuild
with (application / 'Contents/Info.plist').open('rb') as stream:
    info = plistlib.load(stream)
icon_name = info['CFBundleIconFile']
if not Path(icon_name).suffix:
    icon_name += '.icns'
icon = str(application / 'Contents/Resources' / icon_name)
format = 'UDZO'
files = [str(application)]
symlinks = {'Applications': '/Applications'}
hide_extension = [application.name]
background = 'builtin-arrow'
window_rect = ((180, 160), (640, 280))
icon_locations = {application.name: (140, 120), 'Applications': (500, 120)}
icon_size = 96
text_size = 13
label_pos = 'bottom'
show_status_bar = False
show_tab_view = False
show_toolbar = False
show_pathbar = False
show_sidebar = False
show_icon_preview = False
default_view = 'icon-view'
arrange_by = None
