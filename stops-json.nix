{ stdenv }:
stdenv.mkDerivation {
  pname = "stops-json";
  version = "0";
  src = ./stops.json;
  unpackPhase = ''
    for srcFile in $src; do
      cp $srcFile $(stripHash $srcFile)
    done
  '';
  installPhase = ''
    mkdir $out
    cp stops.json $out/stops.json
  '';
}
