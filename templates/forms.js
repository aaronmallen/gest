// gest — form interaction helpers
"use strict";

// -- Markdown Preview Toggle --
(function () {
  document.querySelectorAll(".md-preview-btn").forEach(function (btn) {
    var targetId = btn.dataset.target;
    var textarea = document.getElementById(targetId);
    var preview = document.getElementById(targetId + "-preview");
    if (!textarea || !preview) return;

    btn.addEventListener("click", function (e) {
      e.preventDefault();
      if (preview.classList.contains("active")) {
        preview.classList.remove("active");
        textarea.style.display = "";
        btn.textContent = "Preview";
      } else {
        fetch("/api/render-markdown", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ body: textarea.value })
        })
          .then(function (r) { return r.text(); })
          .then(function (html) {
            preview.innerHTML = html;
            preview.classList.add("active");
            textarea.style.display = "none";
            btn.textContent = "Edit";
            if (window.__renderMermaid) window.__renderMermaid(preview);
          });
      }
    });
  });
})();

// -- Relationships Modal --
(function () {
  var modal = document.getElementById("rel-modal");
  if (!modal) return;

  var toggleBtn = document.getElementById("rel-toggle");
  var typeButtons = modal.querySelectorAll(".rel-type-btn");
  var searchInput = modal.querySelector(".rel-search-input");
  var resultsDiv = modal.querySelector(".rel-results");
  var pendingDiv = modal.querySelector(".rel-pending");
  var hiddenContainer = modal.querySelector(".rel-hidden-inputs");
  var selectedType = "relates-to";
  var debounceTimer = null;

  toggleBtn.setAttribute("aria-expanded", "false");
  toggleBtn.setAttribute("aria-controls", "rel-modal");
  modal.setAttribute("role", "dialog");
  modal.setAttribute("aria-label", "Add relationship");

  function openModal() {
    modal.classList.add("open");
    toggleBtn.setAttribute("aria-expanded", "true");
    searchInput.focus();
    document.addEventListener("keydown", modalKeyHandler);
  }

  function closeModal() {
    modal.classList.remove("open");
    toggleBtn.setAttribute("aria-expanded", "false");
    document.removeEventListener("keydown", modalKeyHandler);
    toggleBtn.focus();
  }

  function getFocusableElements() {
    return modal.querySelectorAll(
      'button, [href], input:not([type="hidden"]), select, textarea, [tabindex]:not([tabindex="-1"])'
    );
  }

  function modalKeyHandler(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeModal();
      return;
    }
    if (e.key === "Tab") {
      var focusable = getFocusableElements();
      if (focusable.length === 0) return;
      var first = focusable[0];
      var last = focusable[focusable.length - 1];
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }

  toggleBtn.addEventListener("click", function (e) {
    e.preventDefault();
    if (modal.classList.contains("open")) {
      closeModal();
    } else {
      openModal();
    }
  });

  typeButtons.forEach(function (btn) {
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      typeButtons.forEach(function (b) { b.classList.remove("active"); });
      btn.classList.add("active");
      selectedType = btn.dataset.rel;
    });
  });

  typeButtons.forEach(function (btn) {
    if (btn.dataset.rel === selectedType) btn.classList.add("active");
  });

  searchInput.addEventListener("input", function () {
    clearTimeout(debounceTimer);
    var q = searchInput.value.trim();
    if (q.length < 2) {
      resultsDiv.innerHTML = "";
      return;
    }
    debounceTimer = setTimeout(function () {
      fetch("/api/search?q=" + encodeURIComponent(q))
        .then(function (r) { return r.json(); })
        .then(function (items) {
          resultsDiv.innerHTML = "";
          items.forEach(function (item) {
            var btn = document.createElement("button");
            btn.type = "button";
            btn.className = "rel-result-item";
            btn.innerHTML = '<span class="c-dim">' + item.type + "</span>" +
              '<span class="c-azure">' + item.short_id + "</span> " + escapeHtml(item.title);
            btn.addEventListener("click", function () {
              addPendingLink(selectedType, item.type, item.id, item.short_id, item.title);
              resultsDiv.innerHTML = "";
              searchInput.value = "";
              searchInput.focus();
            });
            resultsDiv.appendChild(btn);
          });
        });
    }, 300);
  });

  function addPendingLink(rel, entityType, id, shortId, title) {
    var ref = (entityType === "task" ? "tasks/" : "artifacts/") + id;
    if (hiddenContainer.querySelector('input[value="' + ref + '"]')) return;

    var item = document.createElement("div");
    item.className = "rel-pending-item";
    item.innerHTML =
      '<span class="c-pewter">' + rel + "</span> " +
      '<span class="c-dim">' + entityType + "</span> " +
      '<span class="c-azure">' + shortId + "</span> " +
      escapeHtml(title) +
      ' <button type="button" class="rel-remove">&times;</button>';

    item.querySelector(".rel-remove").addEventListener("click", function () {
      item.remove();
      removeHiddenInputs(ref, rel);
    });

    pendingDiv.appendChild(item);

    var relInput = document.createElement("input");
    relInput.type = "hidden";
    relInput.name = "link_rel[]";
    relInput.value = rel;
    hiddenContainer.appendChild(relInput);

    var refInput = document.createElement("input");
    refInput.type = "hidden";
    refInput.name = "link_ref[]";
    refInput.value = ref;
    hiddenContainer.appendChild(refInput);
  }

  function removeHiddenInputs(ref, rel) {
    var rels = hiddenContainer.querySelectorAll('input[name="link_rel[]"]');
    var refs = hiddenContainer.querySelectorAll('input[name="link_ref[]"]');
    for (var i = refs.length - 1; i >= 0; i--) {
      if (refs[i].value === ref && rels[i].value === rel) {
        refs[i].remove();
        rels[i].remove();
        break;
      }
    }
  }

  pendingDiv.querySelectorAll(".rel-pending-item").forEach(function (item) {
    var removeBtn = item.querySelector(".rel-remove");
    if (!removeBtn) return;
    removeBtn.addEventListener("click", function () {
      var relInput = item.querySelector('input[name="link_rel[]"]');
      var refInput = item.querySelector('input[name="link_ref[]"]');
      if (relInput) relInput.remove();
      if (refInput) refInput.remove();
      item.remove();
    });
  });

  function escapeHtml(str) {
    var div = document.createElement("div");
    div.appendChild(document.createTextNode(str));
    return div.innerHTML;
  }
})();
