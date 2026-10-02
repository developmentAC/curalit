// Server-side file/directory chooser: buttons with data-browse="file|dir" fill the target text input.
(function () {
  const modal = document.getElementById('fb-modal');
  const list = document.getElementById('fb-list');
  const pathEl = document.getElementById('fb-path');
  const errEl = document.getElementById('fb-error');
  const selectBtn = document.getElementById('fb-select');
  let target = null, mode = 'file', current = '', selected = null;

  async function load(path) {
    errEl.textContent = '';
    const r = await fetch('/api/browse?path=' + encodeURIComponent(path || ''));
    const j = await r.json();
    if (!r.ok) { errEl.textContent = j.error || 'Error'; return; }
    current = j.path;
    selected = mode === 'dir' ? current : null;
    pathEl.textContent = current;
    selectBtn.disabled = !selected;
    list.replaceChildren();
    if (j.parent) list.append(item('📁 ..', () => load(j.parent)));
    for (const e of j.entries) {
      if (e.is_dir) {
        list.append(item('📁 ' + e.name, () => load(e.path)));
      } else if (mode === 'file') {
        const li = item('📄 ' + e.name, () => {
          list.querySelectorAll('.sel').forEach(x => x.classList.remove('sel'));
          li.classList.add('sel');
          selected = e.path;
          selectBtn.disabled = false;
        });
        list.append(li);
      }
    }
  }

  function item(text, onClick) {
    const li = document.createElement('li');
    li.textContent = text;
    li.addEventListener('click', onClick);
    return li;
  }

  document.querySelectorAll('[data-browse]').forEach(btn => {
    btn.addEventListener('click', () => {
      target = document.getElementById(btn.dataset.target);
      mode = btn.dataset.browse;
      modal.hidden = false;
      selectBtn.textContent = mode === 'dir' ? 'Select this directory' : 'Select file';
      load(target.value);
    });
  });
  selectBtn.addEventListener('click', () => {
    if (target && selected) target.value = selected;
    modal.hidden = true;
  });
  document.getElementById('fb-cancel').addEventListener('click', () => { modal.hidden = true; });
})();
