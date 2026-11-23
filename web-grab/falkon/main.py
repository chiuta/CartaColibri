# MIT — CartaColibri web-grab plugin TEMPLATE for Falkon (PyFalkon).
#
# STATUS: template. It follows Falkon's documented Python-plugin layout (metadata.desktop +
# main.py exposing a Falkon.PluginInterface), but it has NOT been run against a real Falkon build yet.
# Check the hook names marked "VERIFY" against your Falkon version (`falkon --version`, PyFalkon docs).
#
# What it does: adds "Save to CartaColibri" to the page context menu / Tools menu. The page text (or the
# selection) is POSTed to the local CartaColibri server (`cartacolibri serve`), which stores it as Markdown
# with the URL as its source. Nothing leaves your machine.
#
# Install: copy this folder to  ~/.config/falkon/plugins/cartacolibri-webgrab/  (portable build: the
# profile's plugins/ folder), enable it in Falkon > Preferences > Extensions.
import json
import os
import urllib.request

import Falkon  # provided by Falkon's embedded Python
from PySide6 import QtCore, QtGui, QtWidgets

PORT = int(os.environ.get("CARTACOLIBRI_PORT", "8765"))
HOME = os.environ.get("CARTACOLIBRI_HOME") or os.path.join(os.path.expanduser("~"), ".cartacolibri")


def token():
    with open(os.path.join(HOME, "clip.token"), encoding="utf-8") as f:
        return f.read().strip()


def send(url, title, text):
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/carta/clip?token={token()}",
        data=json.dumps({"url": url, "title": title, "text": text}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)


class WebGrab(Falkon.PluginInterface, QtCore.QObject):
    def init(self, state, settingsPath):  # VERIFY signature
        self.action = QtGui.QAction("Save to CartaColibri")
        self.action.triggered.connect(self.grab)
        Falkon.MainApplication.instance().mainWindowCreated.connect(self.on_window)  # VERIFY signal name

    def on_window(self, window):
        window.menuBar().addAction(self.action)  # VERIFY: or a Tools-menu entry via window.menuTools()

    def grab(self):
        view = Falkon.MainApplication.instance().getWindow().weView()  # VERIFY
        page = view.page()

        def done(text):
            try:
                sel = page.selectedText()
                r = send(view.url().toString(), view.title(), sel or text)
                QtWidgets.QMessageBox.information(None, "CartaColibri", f"Saved ({r.get('chars', '?')} characters).")
            except Exception as e:  # server not running, bad token, ...
                QtWidgets.QMessageBox.warning(None, "CartaColibri", f"Could not save: {e}\nIs `cartacolibri serve` running?")

        page.toPlainText(done)

    def unload(self):
        pass

    def testPlugin(self):
        return True


Falkon.registerPlugin(WebGrab())
