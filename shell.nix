{
  pkgs ? import <nixpkgs> { },
}:

let
  libs = with pkgs; [
    alsa-lib
    fontconfig
    libxkbcommon
    xorg.libXcursor
    xorg.libX11
  ];
in
pkgs.mkShell rec {
  packages =
    with pkgs;
    [
      pkg-config
    ]
    ++ libs;

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
}
