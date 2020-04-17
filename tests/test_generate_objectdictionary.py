from pathlib import Path

from tools.generate_objectdictionary import generate

EDS_PATH = Path(__file__).parent / 'sample.eds'


def test_generate(tmp_path):
    output_file_path = tmp_path / 'gen_od.rs'
    od = generate(EDS_PATH, output_file_path, node_id=2)
    print(od)
    print(output_file_path.read_text())
