This program doesn't work yet so don't try to use it:

# this is outdated, I switched to a config file 
# which I will document sooner or later
# once everthing works properly.
This program sends state files back and forth at startup
and post modificatoin for zathura pdf reader from a 
disposable VM to a non-disposable VM. 

setup includes setting three environment variables. 

- client:
  - ZATHURA_BMARK_VM="virtual machine name" : indicates which vm to use 
  - ZBMARK_MODEL="client" : tells the prog. this is the client

- server:
  - ZBMARK_MODEL="server" : tells the prog. this is the server

then you just have to have make you policy.d file on dom0 
and then install your files into their respective locations 
on each virtual machine.

- client:
the executable should go in a system install location like
usr/local/bin or /usr/bin an xdg_autostart desktop file or
something equivalent to this will get the job done for 
starting the program.

- server:
just put the executable inside the /etc/qubes-rpc directory
and set the permissions to match the other qubes services in the dir.
