{
  pkgs ? import <nixpkgs> { },
}:

let
  libs = with pkgs; [
    alsa-lib
    fontconfig
    libX11
    libXcursor
    libxkbcommon
  ];
in
pkgs.mkShell rec {
  packages =
    with pkgs;
    [
      pkg-config
      rustup
    ]
    ++ libs;

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
}
