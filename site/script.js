// Copy-to-clipboard for the install code blocks (PxCodeBlock port).
(function () {
  function setLabel(button, text) {
    button.textContent = text;
    button.setAttribute('aria-label', text === 'Copied' ? 'Copied command' : 'Copy command');
  }

  document.querySelectorAll('[data-copy]').forEach(function (button) {
    button.addEventListener('click', function () {
      var target = document.getElementById(button.getAttribute('data-copy'));
      if (!target) return;
      var text = target.textContent;
      function done(ok) {
        setLabel(button, ok ? 'Copied' : 'Copy');
        if (ok) window.setTimeout(function () { setLabel(button, 'Copy'); }, 1600);
      }
      function legacyCopy() {
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.setAttribute('readonly', '');
        ta.style.position = 'fixed';
        ta.style.opacity = '0';
        document.body.appendChild(ta);
        ta.select();
        var ok = false;
        try { ok = document.execCommand('copy'); } catch (e) { ok = false; }
        document.body.removeChild(ta);
        return ok;
      }
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(function () { done(true); }, function () { done(legacyCopy()); });
      } else {
        done(legacyCopy());
      }
    });
  });
})();
