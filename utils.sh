# Create a macOS app bundle in the 'bin' directory.
function create_app_bundle {
    echo "Making bin/Limit Theory Redux.app..." 

    local destination
    destination="bin"

    mkdir "$destination/Limit Theory Redux.app/"
    mkdir "$destination/Limit Theory Redux.app/Contents/"
    mkdir "$destination/Limit Theory Redux.app/Contents/Resources/"
    mkdir "$destination/Limit Theory Redux.app/Contents/MacOS/"
    cp "res/images/LTR-Icon.icns" "$destination/Limit Theory Redux.app/Contents/Resources/ltr.icns"

    cat > "$destination/Limit Theory Redux.app/Contents/Info.plist" << EOF
    {
        CFBundleName = "Limit Theory Redux";
        CFBundleDisplayName = "Limit Theory Redux";
        CFBundleIdentifier = "org.LimitTheoryRedux.LimitTheoryRedux";
        CFBundleVersion = "$PHX_VERSION";
        CFBundleShortVersionString = "$PHX_VERSION";
        CFBundleInfoDictionaryVersion = "6.0";
        CFBundlePackageType = APPL;
        CFBundleSignature = ltrr;
        CFBundleExecutable = ltr;
        CFBundleIconFile = "ltr.icns";
    }
EOF

    echo "Copying data files ..."
    rsync -Ca ./res "$destination/Limit Theory Redux.app/Contents/"
    rsync -Ca ./script "$destination/Limit Theory Redux.app/Contents/"
    mkdir -p "$destination/Limit Theory Redux.app/Contents/engine/lib/phx"
    rsync -Ca ./engine/lib/phx/script "$destination/Limit Theory Redux.app/Contents/engine/lib/phx"

    echo "Copying binary ..."
    cp -a "${target_dir}/ltr${binsuffix}" "$destination/Limit Theory Redux.app/Contents/MacOS/ltr"

    echo "Copying and fixing dynamic libraries... "
    cp "${target_dir}/deps/${libprefix}phx${libsuffix}" "$destination/Limit Theory Redux.app/Contents/MacOS/${libprefix}phx${libsuffix}"

    echo "Re-sign libraries with an 'ad-hoc signing' see man codesign"
    # codesign --sign - --force $destination/Limit Theory Redux.app/Contents/libs/*

    echo "Stripping binary ..."
    strip -u -r "$destination/Limit Theory Redux.app/Contents/MacOS/ltr"
}