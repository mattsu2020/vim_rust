README.txt for the gvimext DLL.

Written by Tianmiao Hu.  Edited by Bram Moolenaar.

This extension is now implemented in Rust as the `rust_gvimext` crate.
The original C++ sources have been removed.


INSTALLATION

To install the "Edit with Vim" popup menu entry, it is recommended to use the
"install.exe" program.  It will ask you a few questions and install the needed
registry entries.

In special situations you might want to make changes by hand.  Check these
items:
- The gvimext.dll, gvim.exe and uninstall.exe either need to be in the search
  path, or you have to set the full path in the registry entries.  You could
  move the gvimext.dll to the "windows\system" or "windows\system32"
  directory, where the other DLL files are.
- You can find the names of the used registry entries in the file
  "GvimExt.reg".  You can edit this file to add the paths.  To install the
  registry entries, right-click the gvimext.reg file and choose the "merge"
  menu option.
- The registry key [HKEY_LOCAL_MACHINE\Software\Vim\Gvim] is used by the
  gvimext.dll.  The value "path" specifies the location of "gvim.exe".  If
  gvim.exe is in the search path, the path can be omitted.  The value "lang"
  can be used to set the language, for example "de" for German.  If "lang" is
  omitted, the language set for Windows will be used.

It is the preferred method to keep gvim.exe with the runtime files, so that
Vim will find them (also the translated menu items are there).


UNINSTALLATION

To uninstall the "Edit with Vim" popup menu entry, it is recommended to use
the "uninstal.exe" program.

In special situations you might want to uninstall by hand:
- Open the registry by running regedit.exe.
- Delete all the keys listed in GvimExt.reg, except this one:
  [HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Approved]
  For this key, only delete one value:
  "{51EEE242-AD87-11d3-9C1E-0090278BBD99}"="Vim Shell Extension"
- Delete the gvimext.dll, if you want.  You might need to reboot the machine
  in order to remove this file.  A quick way is to log off and re-login.

Another method is by using the uninst.bat script:
    uninst gvimext.inf
This batch file will remove all the registry keys from the system.  Then you
can remove the gvimext.dll file.
Note: In order for this batch file to work, you must have two system files:
rundll32.exe and setupapi.dll.  I believe you will have rundll32.exe in your
system.  I know windows nt 4.0 with the service pack 4 has setupapi.dll.  My
windows 95 has setupapi.dll.  I find that the internet explorer 4.0 comes with
the setupapi.dll in file Ie4_5.cab.

If you do encounter problems running this script, then probably you need to
modify the uninst.bat to suit to your system.  Basically, you must find out
where are the locations for your rundll32.exe and setupapi.dll files.  In
windows nt, both files are under c:\winnt\system32 directory. In my windows 95
system, I got setupapi.dll at c:\windows\system and rundll32.exe at
c:\windows.  So you might want to try something like:
    rundll32.exe c:\windows\system\setupapi.dll,InstallHinfSection DefaultUninstall 128 %1
where %1 can be substituted by gvimext.inf


THE SOURCE CODE

The shell extension is now written in Rust.  The implementation lives in the
`rust_gvimext` crate and is built with Cargo.

Happy vimming!!!
