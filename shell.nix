{
  pkgs ? import <nixpkgs> { },
}:

let
  libs = with pkgs; [
    alsa-lib
    libGL
    libx11
    libxcursor
    libxi
    libxkbcommon
    libxrandr
    wayland
  ];
in
pkgs.mkShell rec {
  packages =
    with pkgs;
    [
      pkg-config
      gnuplot
      rustup
    ]
    ++ libs;

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
}
