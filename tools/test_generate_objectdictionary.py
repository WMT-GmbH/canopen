from pathlib import Path

from tools.generate_objectdictionary import generate

EDS_PATH = Path(__file__).parent / 'sample.eds'
OUT_PATH = Path(__file__).parent / 'gen_od.rs'


def test_generate(tmp_path):
    output_file_path = tmp_path / 'gen_od.rs'
    output_file_path = OUT_PATH
    generate(EDS_PATH, output_file_path, node_id=2)
    print(output_file_path.read_text())
