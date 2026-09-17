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
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(function () { done(true); }, function () { done(false); });
      } else {
        var ta = document.createElement('textarea');
        ta.value = text;
        document.body.appendChild(ta);
        ta.select();
        try { done(document.execCommand('copy')); } catch (e) { done(false); }
        document.body.removeChild(ta);
      }
    });
  });
})();
