# /bin/bash!
#
# Script for automated testing of 
# qubes rpc service programs functionalities
# Change these if personal values differ:
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ #
LOCAL_SCRIPT_PATH="/home/shannel/debug_scripts/split_zathura/zath_bmark_vm_build.sh";
QINC_SCRIPT_PATH="/QubesIncoming/dom0/zath_bmark_vm_build.sh";

COMPILER_VM="dev";
COMPILER_VM_USER="user";
PROJECT_DIR="/home/$COMPILER_VM_USER/source/rust/qubes-zathura-bookmark";
ZIP_FNAME="qzb.zip";

SERVER_VM_USER="user";
SERVER_QINC="/home/$SERVER_VM_USER/QubesIncoming/dom0";
SERVER_ZIP_PATH="$SERVER_QINC/$ZIP_FNAME";
SERVER_SCRIPT_PATH="$SERVER_QINC/zath_bmark_vm_build.sh";
SERVER_VM="zstate-server";
SERVER_PKG_NAME="qubes-zathura-bookmark";

CLIENT_VM_USER="user";
CLIENT_QINC="/home/$CLIENT_VM_USER/QubesIncoming/dom0";
CLIENT_ZIP_PATH="$CLIENT_QINC/$ZIP_FNAME";
CLIENT_SCRIPT_PATH="$CLIENT_QINC/zath_bmark_vm_build.sh";
CLIENT_VM="reader";
CLIENT_PKG_NAME="qubes-zathura-bookmark";
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ #

WORKING_DIR="$(mktemp --directory)" || exit 1;

qvm-run --pass-io -u $COMPILER_VM_USER $COMPILER_VM "
cd $PROJECT_DIR &&
zip -r $ZIP_FNAME ./* -x 'target/***' '.git/***' '.gitignore' \
'zath_bmark_debug.sh' 'zath_bmark_vm_build.sh' 'LICENSE.md' 'README.md' &&
cat $ZIP_FNAME" > $WORKING_DIR/$ZIP_FNAME || exit 2;
qvm-run $COMPILER_VM "rm -f $PROJECT_DIR/$ZIP_FNAME" || exit 3;

qvm-run -u $CLIENT_VM_USER $CLIENT_VM "
rm -f $CLIENT_ZIP_PATH $CLIENT_SCRIPT_PATH" || exit 8; 
qvm-copy-to-vm $CLIENT_VM $WORKING_DIR/$ZIP_FNAME || exit 4;
qvm-copy-to-vm $CLIENT_VM $LOCAL_SCRIPT_PATH || exit 5;

qvm-run -u $SERVER_VM_USER  $SERVER_VM "
rm -f $SERVER_ZIP_PATH $SERVER_SCRIPT_PATH" || exit 9;
qvm-copy-to-vm $SERVER_VM $WORKING_DIR/$ZIP_FNAME || exit 6;
qvm-copy-to-vm $SERVER_VM $LOCAL_SCRIPT_PATH || exit 7; 

qvm-run -u root --pass-io $CLIENT_VM "
$CLIENT_SCRIPT_PATH $CLIENT_PKG_NAME $CLIENT_VM_USER 0" || exit 8;

qvm-run -u root --pass-io $SERVER_VM "
$SERVER_SCRIPT_PATH $SERVER_PKG_NAME $SERVER_VM_USER 1" || exit 9;

rm -rf $WORKING_DIR &> /dev/null || exit 10; 

exit 0;
