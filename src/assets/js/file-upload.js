(function () {
  function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + ' بایت';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' کیلوبایت';
    return (bytes / (1024 * 1024)).toFixed(2) + ' مگابایت';
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
    if (!fileInput) return;

    fileInput.addEventListener('change', function () {
      updateFileUi(fileInput, iconEl, promptEl, nameEl, sizeEl, rowEl, cardEl, zoneEl, clearBtn);
    });

    if (clearBtn) {
      clearBtn.addEventListener('click', function (e) {
        e.preventDefault();
        e.stopPropagation();
        fileInput.value = '';
        updateFileUi(fileInput, iconEl, promptEl, nameEl, sizeEl, rowEl, cardEl, zoneEl, clearBtn);
      });
    }
  });
})();
