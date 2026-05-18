(function () {
  function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + ' بایت';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' کیلو';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' مگ';
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' گیگ';
  }

  function setHidden(el, hidden) {
    if (!el) return;
    el.hidden = hidden;
  }

  function updateFileUi(fileInput, iconEl, promptEl, nameEl, sizeEl, rowEl, cardEl, zoneEl, clearBtn) {
    var hasFile = fileInput.files && fileInput.files.length > 0;

    if (cardEl) cardEl.classList.toggle('has-file', hasFile);
    if (zoneEl) zoneEl.classList.toggle('has-file', hasFile);

    setHidden(iconEl, hasFile);
    setHidden(promptEl, hasFile);
    setHidden(rowEl, !hasFile);
    setHidden(clearBtn, !hasFile);

    if (hasFile) {
      var file = fileInput.files[0];
      if (nameEl) nameEl.textContent = file.name;
      if (sizeEl) sizeEl.textContent = '(' + formatFileSize(file.size) + ')';
    } else {
      if (nameEl) nameEl.textContent = '';
      if (sizeEl) sizeEl.textContent = '';
    }

    document.activeElement.blur();
  }

  function assignFileToInput(fileInput, file) {
    if (!fileInput || !file) return;
    var dt = new DataTransfer();
    dt.items.add(file);
    fileInput.files = dt.files;
    fileInput.dispatchEvent(new Event('change', { bubbles: true }));
  }

  function getClipboardFile(clipboardData) {
    if (!clipboardData || !clipboardData.items) return null;
    for (var i = 0; i < clipboardData.items.length; i++) {
      var item = clipboardData.items[i];
      if (item.kind === 'file') {
        return item.getAsFile();
      }
    }
    return null;
  }

  function shouldHandlePaste(e) {
    if (!getClipboardFile(e.clipboardData)) return false;

    var target = e.target;
    if (!target || !target.tagName) return true;

    var tag = target.tagName;
    var isTextField =
      tag === 'TEXTAREA' ||
      (tag === 'INPUT' &&
        (target.type === 'text' ||
          target.type === 'password' ||
          target.type === 'search' ||
          target.type === 'email' ||
          target.type === 'url'));

    if (!isTextField) return true;

    var text = e.clipboardData.getData('text/plain');
    return !text || !text.length;
  }

  function isFileDrag(e) {
    var dt = e.dataTransfer;
    if (!dt || !dt.types) return false;
    if (typeof dt.types.contains === 'function') return dt.types.contains('Files');
    return Array.prototype.indexOf.call(dt.types, 'Files') !== -1;
  }

  function revealShareFormIfNeeded() {
    var shareForm = document.getElementById('shareForm');
    var home = document.querySelector('.home');
    if (!shareForm || !home) return;

    var visible =
      shareForm.style.display === 'flex' || window.getComputedStyle(shareForm).display === 'flex';
    if (visible) return;

    home.style.display = 'none';
    shareForm.style.display = 'flex';
  }

  function setupDragAndDrop(fileInput, fileWrapper, refreshUi) {
    var dragDepth = 0;

    function setDragActive(active) {
      if (fileWrapper) fileWrapper.classList.toggle('file-drag-active', active);
      document.body.classList.toggle('file-page-drag', active);
    }

    function resetDragState() {
      dragDepth = 0;
      setDragActive(false);
    }

    window.addEventListener(
      'dragenter',
      function (e) {
        if (!isFileDrag(e)) return;
        e.preventDefault();
        dragDepth += 1;
        setDragActive(true);
      },
      false
    );

    window.addEventListener(
      'dragover',
      function (e) {
        if (!isFileDrag(e)) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = 'copy';
      },
      false
    );

    window.addEventListener(
      'dragleave',
      function (e) {
        if (!isFileDrag(e)) return;
        dragDepth -= 1;
        if (dragDepth <= 0) resetDragState();
      },
      false
    );

    window.addEventListener(
      'dragend',
      function () {
        resetDragState();
      },
      false
    );

    window.addEventListener(
      'drop',
      function (e) {
        if (!isFileDrag(e)) return;
        e.preventDefault();
        resetDragState();

        var files = e.dataTransfer.files;
        if (!files || !files.length) return;

        revealShareFormIfNeeded();
        assignFileToInput(fileInput, files[0]);
        refreshUi();
      },
      false
    );
  }

  function setupPaste(fileInput, refreshUi) {
    document.addEventListener(
      'paste',
      function (e) {
        if (!shouldHandlePaste(e)) return;

        var file = getClipboardFile(e.clipboardData);
        if (!file) return;

        e.preventDefault();
        revealShareFormIfNeeded();
        assignFileToInput(fileInput, file);
        refreshUi();
      },
      false
    );
  }

  document.addEventListener('DOMContentLoaded', function () {
    var fileInput = document.getElementById('file');
    var iconEl = document.getElementById('fileUploadIcon');
    var promptEl = document.getElementById('fileUploadPrompt');
    var nameEl = document.getElementById('fileSelectedName');
    var sizeEl = document.getElementById('fileSelectedSize');
    var rowEl = document.getElementById('fileSelectedRow');
    var cardEl = document.getElementById('fileUploadCard');
    var zoneEl = document.getElementById('fileUploadZone');
    var clearBtn = document.getElementById('fileClearBtn');
    var fileWrapper = document.getElementById('fileWrapper');
    if (!fileInput) return;

    function refreshUi() {
      updateFileUi(fileInput, iconEl, promptEl, nameEl, sizeEl, rowEl, cardEl, zoneEl, clearBtn);
    }

    fileInput.addEventListener('change', refreshUi);

    if (clearBtn) {
      clearBtn.addEventListener('click', function (e) {
        e.preventDefault();
        e.stopPropagation();
        fileInput.value = '';
        refreshUi();
      });
    }

    setupDragAndDrop(fileInput, fileWrapper, refreshUi);
    setupPaste(fileInput, refreshUi);
  });
})();
