#!/usr/bin/env bash
set -euo pipefail
# Run as root with uv available. Download only this pinned checkpoint, then run offline.
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
revision=7b928d828b7b0e022f929d9bd2e44165aa270148
getent passwd khata-laya >/dev/null || useradd --system --home-dir /var/lib/khata-laya --shell /usr/sbin/nologin khata-laya
if systemctl is-active --quiet khata-laya.service; then
    systemctl stop khata-laya.service
fi
install -d -m 755 /opt/khata-laya
uv venv --allow-existing --python /usr/bin/python3.13 /opt/khata-laya/venv
uv pip install --python /opt/khata-laya/venv/bin/python --extra-index-url https://download.pytorch.org/whl/cpu --index-strategy unsafe-best-match -r "$here/requirements.txt"
chmod -R a+rX /opt/khata-laya/venv
install -d -o khata-laya -g khata-laya -m 700 /var/lib/khata-laya
install -d -o khata-laya -g khata-laya -m 700 /var/lib/khata-laya/model
KHATA_MODEL_REVISION="$revision" /opt/khata-laya/venv/bin/python - <<'PY'
import os, shutil
from pathlib import Path
from huggingface_hub import snapshot_download
source = Path(snapshot_download('convaiinnovations/laya',revision=os.environ['KHATA_MODEL_REVISION'],
    allow_patterns=['config.json','rl_agent_config.json','model.safetensors','encoder/config.json','tokenizer/*']))
target = Path('/var/lib/khata-laya/model')
for path in source.rglob('*'):
    if path.is_file():
        dest=target/path.relative_to(source)
        dest.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(path,dest)
PY
chown -R khata-laya:khata-laya /var/lib/khata-laya
install -m 644 "$here/sms_service.py" /opt/khata-laya/sms_service.py
install -m 644 "$here/sms_policy.py" /opt/khata-laya/sms_policy.py
install -m 644 "$here/khata-laya.service" /etc/systemd/system/khata-laya.service
systemctl daemon-reload
systemctl enable khata-laya.service
systemctl restart khata-laya.service
/opt/khata-laya/venv/bin/python - <<'PY'
import time, urllib.request
deadline = time.monotonic() + 90
while time.monotonic() < deadline:
    try:
        with urllib.request.urlopen('http://127.0.0.1:8091/health', timeout=2) as response:
            if response.status == 200:
                print('Laya service ready on loopback')
                break
    except OSError:
        time.sleep(1)
else:
    raise SystemExit('Laya service did not become ready; inspect khata-laya.service')
PY
