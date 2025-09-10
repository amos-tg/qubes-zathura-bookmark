#! /bin/bash
#
# 
#
# # # # # # # # # # # # # # # 

# Script Arguments #
PKG_NAME="$1";
USER_U="$2";
IS_SERVER="$3"; 
# ~~~~~~~~~~~~~~~~ #

HOME_U="/home/$USER_U";
QINC_DIR="$HOME_U/QubesIncoming/dom0";
SELF_PATH="$QINC_DIR/zath_bmark_vm_build.sh";
ZIP_PATH="$QINC_DIR/qzb.zip";
PROJ_DIR="$HOME_U/qzb";
PROJ_MANIFEST="$PROJ_DIR/Cargo.toml";
AOUT="$PROJ_DIR/target/debug/$PKG_NAME";
VAULT_RPC_PATH="/etc/qubes-rpc/qubes.ZathuraMgmt";
# ~~~~~~~~~~~~~~~~ #

shopt -s extglob;

if [ -d $PROJ_DIR ]; then
  rm -rf $PROJ_DIR/!(*target);
else
  mkdir $PROJ_DIR; fi

unzip $ZIP_PATH -d $PROJ_DIR || true;
cargo build --manifest-path $PROJ_MANIFEST || exit 2;
chown root:root $AOUT || exit 5;
chmod 755 $AOUT || exit 4

if [ $IS_SERVER == 1 ]; then 
  mv $AOUT $VAULT_RPC_PATH || exit 7; 
fi

rm -f $SELF_PATH || exit 9;
