{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "h264 nal unit parser exercise";

  buildInputs = with pkgs; [
    # todo!()
    # gst_all_1.gstreamer
    # gst_all_1.gst-plugins-base
    # gst_all_1.gst-plugins-good
    # gst_all_1.gst-plugins-bad
    # gst_all_1.gst-plugins-ugly
    # gst_all_1.gst-libav

    pkg-config
    rustup
  ];

  shellHook = ''
    rustup default stable
    echo "basic development shell for Rust and GStreamer is ready!"
  '';
}

