' Starts the CartaColibri server without a console window, then opens the browser.
' Usage: wscript CartaColibri-hidden.vbs [browser]
Option Explicit
Dim sh, fso, here, home, app, port, url, exe
Set sh = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
here = fso.GetParentFolderName(WScript.ScriptFullName)
If sh.ExpandEnvironmentStrings("%CARTACOLIBRI_HOME%") = "%CARTACOLIBRI_HOME%" Then
  If fso.FolderExists(here & "\Data") Then
    home = here & "\Data\cartacolibri"
  Else
    home = sh.ExpandEnvironmentStrings("%USERPROFILE%") & "\.cartacolibri"
  End If
Else
  home = sh.ExpandEnvironmentStrings("%CARTACOLIBRI_HOME%")
End If
If Not fso.FolderExists(fso.GetParentFolderName(home)) Then fso.CreateFolder fso.GetParentFolderName(home)
If Not fso.FolderExists(home) Then fso.CreateFolder home
sh.Environment("PROCESS")("CARTACOLIBRI_HOME") = home
app = here & "\App\CartaColibri\"
port = "8765"
url = "http://127.0.0.1:" & port & "/"
If WScript.Arguments.Count > 0 Then
  If LCase(WScript.Arguments(0)) = "browser" Then
    sh.Run """" & app & "cartacolibri-browser.exe"" --home """ & home & """", 1, False
    WScript.Quit 0
  End If
End If
If Not Probe(port) Then
  sh.Run """" & app & "cartacolibri.exe"" serve --port " & port, 0, False
  Dim i
  For i = 1 To 50
    If Probe(port) Then Exit For
    WScript.Sleep 200
  Next
End If
sh.Run url, 1, False

Function Probe(p)
  ' true if something already answers on 127.0.0.1:p (local only)
  On Error Resume Next
  Dim x
  Set x = CreateObject("MSXML2.ServerXMLHTTP")
  x.setTimeouts 300, 300, 300, 300
  x.open "GET", "http://127.0.0.1:" & p & "/", False
  x.send
  Probe = (Err.Number = 0)
  On Error GoTo 0
End Function
