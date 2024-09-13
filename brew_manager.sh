#!/bin/bash

command=$1
package=$2

case $command in
    install)
        brew install $package
        ;;
    update)
        brew update $package
        ;;
    uninstall)
        brew uninstall $package
        ;;
    search)
        brew search $package
        ;;
    info)
        brew info $package
        ;;
    list)
        brew list
        ;;
    outdated)
        brew outdated
        ;;
    cleanup)
        brew cleanup
        ;;
    doctor)
        brew doctor
        ;;
    backup)
        # 备份命令
        ;;
    restore)
        # 恢复命令
        ;;
    *)
        echo "未知命令: $command"
        ;;
esac