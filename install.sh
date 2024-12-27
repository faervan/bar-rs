#!/bin/sh

directory=$(dirname $(realpath "$0"))

sed -i "s|project_path=\"\"|project_path=\"$directory\"|" $directory/bar-rs

cp_cmd="cp $directory/bar-rs /usr/local/bin"
chmod_cmd="chmod +x /usr/local/bin/bar-rs"

if [ "$UID" -ne 0 -a "$EUID" -ne 0 ]; then
    sudo $cp_cmd
    sudo $chmod_cmd
else
    $cp_cmd
    $chmod_cmd
fi

sed -i "s|project_path=\"$directory\"|project_path=\"\"|" $directory/bar-rs

echo -e "Uninstall bar-rs by running \`bar-rs uninstall\`\n"
echo You need to build the project before you can open the bar:
echo -e "\`cargo build --release\` to build for release (recommended)"
echo -e "\`cargo build\` to build for debug (not recommended)"

echo Done
