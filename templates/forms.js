// gest — form interaction helpers
"use strict";

(function () {
  // ── Relationships Modal ───────────────────────────────────────────
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

  // Toggle modal open/close
  toggleBtn.addEventListener("click", function (e) {
    e.preventDefault();
    modal.classList.toggle("open");
  });

  // Type selection
  typeButtons.forEach(function (btn) {
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      typeButtons.forEach(function (b) { b.classList.remove("active"); });
      btn.classList.add("active");
      selectedType = btn.dataset.rel;
    });
  });

  // Set initial active type
  typeButtons.forEach(function (btn) {
    if (btn.dataset.rel === selectedType) btn.classList.add("active");
  });

  // Debounced search
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
            var div = document.createElement("div");
            div.className = "rel-result-item";
            div.innerHTML = '<span class="c-dim">' + item.type + "</span>" +
              '<span class="c-azure">' + item.short_id + "</span> " + escapeHtml(item.title);
            div.addEventListener("click", function () {
              addPendingLink(selectedType, item.type, item.id, item.short_id, item.title);
              resultsDiv.innerHTML = "";
              searchInput.value = "";
            });
            resultsDiv.appendChild(div);
          });
        });
    }, 300);
  });

  function addPendingLink(rel, entityType, id, shortId, title) {
    // Avoid duplicates
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

    // Add hidden inputs
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

  // Pre-populate remove buttons for existing links
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
