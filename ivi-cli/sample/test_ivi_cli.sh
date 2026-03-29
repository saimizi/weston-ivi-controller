#!/usr/bin/bash

IVI_CLI=$(which ivi_cli)

SCREEN_NAME="HDMI-A-1"
SCREEN_WIDTH=1920
SCREEN_HEIGHT=1080

do_cleanup() {
	echo "Cleaning up..."
	$IVI_CLI layer destroy 1000
	$IVI_CLI layer destroy 2000
}

get_surface_org() {
	ivi_cli surface get-props $1 2>/dev/null | grep OrigSize | cut -d ':' -f 2 | sed 's/x/ /'
}

$IVI_CLI surface list
SURFACE=$($IVI_CLI surface list --ids-only)

if [ -z "$SURFACE" ]; then
	echo "No surfaces found. Please create a surface before running this script."
	exit 1
fi


do_cleanup

# Create layer 1000
$IVI_CLI layer create 1000 $SCREEN_WIDTH $SCREEN_HEIGHT
$IVI_CLI layer set-src-rect 1000 0 0 $SCREEN_WIDTH $SCREEN_HEIGHT
$IVI_CLI layer set-dest-rect 1000 0 0 $SCREEN_WIDTH $SCREEN_HEIGHT

# Create layer 2000
$IVI_CLI layer create 2000 $SCREEN_WIDTH $SCREEN_HEIGHT
$IVI_CLI layer set-src-rect 2000 0 0 $SCREEN_WIDTH $SCREEN_HEIGHT
$IVI_CLI layer set-dest-rect 2000 0 0 $SCREEN_WIDTH $SCREEN_HEIGHT

# Add layers to screen
$IVI_CLI screen set-layers $SCREEN_NAME 1000,2000

# Attach surface to layer 1000
for surface in $SURFACE
do
	$IVI_CLI layer add-surface 1000 $surface
	$IVI_CLI layer add-surface 2000 $surface
done


# Set layers visibility to true
$IVI_CLI layer set-visibility 1000 true
$IVI_CLI layer set-visibility 2000 true
for surface in $SURFACE
do
	orig_size=$(get_surface_org $surface)
	$IVI_CLI surface set-src-rect $surface 0 0 $orig_size 
	$IVI_CLI surface set-dest-rect $surface 0 0 $SCREEN_WIDTH $SCREEN_HEIGHT
	$IVI_CLI surface set-visibility $surface true
done

$IVI_CLI scene
