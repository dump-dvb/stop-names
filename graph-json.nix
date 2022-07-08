{ stdenv }:
stdenv.mkDerivation {
  pname = "graph-json";
  version = "0";
  src = ./graph.json;
  unpackPhase = ''
    for srcFile in $src; do
      cp $srcFile $(stripHash $srcFile)
    done
  '';
  installPhase = ''
    mkdir $out
    cp graph.json $out/graph.json
  '';
}
