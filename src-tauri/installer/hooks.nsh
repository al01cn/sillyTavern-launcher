    Var DeleteUserData

    !macro NSIS_HOOK_PREUNINSTALL

    !ifdef LANG_SIMPCHINESE
        !define DeleteDataQuestion_CN "是否同时删除所有用户数据？$\n$\n包括：SillyTavern 版本、配置文件、日志等$\n$\n选择【是】将永久删除所有数据（不可恢复）$\n选择【否】将保留数据以便重新安装"
    !endif
    !ifdef LANG_ENGLISH
        !define DeleteDataQuestion_EN "Do you want to delete all user data?$\n$\nIncluding: SillyTavern versions, configs, logs, etc.$\n$\nYes = Delete all data (cannot be undone)$\nNo = Keep data for reinstallation"
    !endif

    !ifdef LANG_SIMPCHINESE
        MessageBox MB_YESNO|MB_ICONQUESTION "${DeleteDataQuestion_CN}" IDYES pre_yes IDNO pre_no
    !else
        MessageBox MB_YESNO|MB_ICONQUESTION "${DeleteDataQuestion_EN}" IDYES pre_yes IDNO pre_no
    !endif

    pre_yes:
        StrCpy $DeleteUserData "1"
        Goto pre_done

    pre_no:
        StrCpy $DeleteUserData "0"

    pre_done:

    !macroend



    !macro NSIS_HOOK_POSTUNINSTALL

    StrCmp $DeleteUserData "1" post_delete post_skip

    post_delete:
        DetailPrint "正在删除用户数据..."
        RMDir /r "$INSTDIR\data"
        DetailPrint "用户数据已删除"
        Goto post_done

    post_skip:
        DetailPrint "已保留用户数据"

    post_done:

    !macroend