#!/bin/sh

option() {
    task_type='install'
    build_projects='3'
    daemon_run='y'
}

tinyproxy_set() {
    tinyproxy_port=''
    tinyproxy_proxy_key='Meng'
    tinyproxy_install_dir='/usr/local/tinyproxy'
    tinyproxy_UPX=''
    export tinyproxy_port tinyproxy_proxy_key tinyproxy_install_dir tinyproxy_UPX
}

cns_set() {
    cns_port=''
    cns_encrypt_password=''
    cns_udp_flag='httpUDP'
    cns_proxy_key='Meng'
    cns_tls_port=''
    cns_install_dir='/usr/local/cns'
    cns_UPX=''
    export cns_port cns_encrypt_password cns_udp_flag cns_proxy_key cns_tls_port cns_install_dir cns_UPX
}

xray_set() {
    xray_install_directory='/usr/local/xray'
    xray_inbounds_options='1'
    vmess_tcp_http_port='8090'
    vmess_tcp_tls_port=''
    vless_tcp_reality_port=''
    vmess_ws_port=''
    vmess_ws_path=''
    vmess_ws_tls_port=''
    vmess_ws_tls_path=''
    vless_ws_tls_port=''
    vless_ws_tls_path=''
    vmess_mkcp_port=''
    vmess_mkcp_tls_port=''
    vless_mkcp_tls_port=''
    trojan_tls_port=''
    xray_UPX=''
    export xray_install_directory xray_inbounds_options vmess_tcp_http_port vmess_tcp_tls_port vless_tcp_reality_port vmess_ws_port vmess_ws_path vmess_ws_tls_port vmess_ws_tls_path vless_ws_tls_port vless_ws_tls_path vmess_mkcp_port vmess_mkcp_tls_port vless_mkcp_tls_port trojan_tls_port xray_UPX
}

amy4Server_set() {
    amy4Server_auth_secret=''
    amy4Server_secret_password=''
    amy4Server_port=''
    amy4Server_clientkey=''
    ipv6_support='false'
    amy4Server_install_dir='/usr/local/amy4Server'
    amy4Server_UPX=''
    export amy4Server_auth_secret amy4Server_secret_password amy4Server_port amy4Server_clientkey ipv6_support amy4Server_install_dir amy4Server_UPX
}

tinyproxy_task() {
    if $download_tool_cmd tinyproxy.sh http://binary.quicknet.cyou/tinyproxy/tinyproxy.sh; then
        chmod 777 tinyproxy.sh
        sed -i "s~#\!/bin/bash~#\!$SHELL~" tinyproxy.sh
        ./tinyproxy.sh $task_type && \
                echo 'tinyproxy任务成功' >>builds.log || \
                echo 'tinyproxy启动失败' >>builds.log
    else
        echo 'tinyproxy脚本下载失败' >>builds.log
    fi
    rm -f tinyproxy.sh
}

cns_task() {
    if $download_tool_cmd cns.sh http://binary.quicknet.cyou/cns/cns.sh; then
        chmod 777 cns.sh
        sed -i "s~#\!/bin/bash~#\!$SHELL~" cns.sh
        echo $echo_opt_e "n\ny\ny\ny\ny\n"|./cns.sh $task_type && \
                echo 'cns任务成功' >>builds.log || \
                echo 'cns启动失败' >>builds.log
    else
        echo 'cns脚本下载失败' >>builds.log
    fi
    rm -f cns.sh
}

xray_task() {
    if $download_tool_cmd xray.sh http://binary.quicknet.cyou/xray/xray.sh; then
        chmod 777 xray.sh
        sed -i "s~#\!/bin/bash~#\!$SHELL~" xray.sh
        echo $echo_opt_e "n\ny\ny\ny\ny\n"|./xray.sh $task_type && \
            echo 'xray任务成功' >>builds.log || \
            echo 'xray任务失败' >>builds.log
    else
        echo 'xray脚本下载失败' >>builds.log
    fi
    rm -f xray.sh
}

amy4Server_task() {
    if $download_tool_cmd amy4Server.sh http://binary.quicknet.cyou/amy4Server/amy4Server.sh; then
        chmod 777 amy4Server.sh
        sed -i "s~#\!/bin/bash~#\!$SHELL~" amy4Server.sh
        echo $echo_opt_e "n"|./amy4Server.sh $task_type && \
            echo 'amy4Server任务成功' >>builds.log || \
            echo 'amy4Server任务失败' >>builds.log
    else
        echo 'amy4Server脚本下载失败' >>builds.log
    fi
    rm -f amy4Server.sh
}

tinyproxy_uninstall_set() {
    tinyproxy_install_dir='/usr/local/tinyproxy'
    export tinyproxy_install_dir
}

cns_uninstall_set() {
    cns_install_dir='/usr/local/cns'
    export cns_install_dir
}

xray_uninstall_set() {
    xray_install_directory='/usr/local/xray'
    export xray_install_directory
}

amy4Server_uninstall_set() {
    amy4Server_install_dir='/usr/local/amy4Server'
    export amy4Server_install_dir
}

server_install_set() {
    for opt in $*; do
        case $opt in
            1) tinyproxy_set;;
            2) cns_set;;
            3) xray_set;;
            4) amy4Server_set;;
            *) exec echo "选项($opt)不正确，请输入正确的选项！";;
        esac
    done
}

server_uninstall_set() {
    for opt in $*; do
        case $opt in
            1) tinyproxy_uninstall_set;;
            2) cns_uninstall_set;;
            3) xray_uninstall_set;;
            4) amy4Server_uninstall_set;;
            *) exec echo "选项($opt)不正确，请输入正确的选项！";;
        esac
    done
}

start_task() {
    for opt in $*; do
        case $opt in
            1) tinyproxy_task;;
            2) cns_task;;
            3) xray_task;;
            4) amy4Server_task;;
        esac
        sleep 1
    done
    echo '所有任务完成' >>builds.log
    echo $echo_opt_e "\033[32m`cat builds.log 2>&-`\033[0m"
}

run_tasks() {
    [ "$task_type" != 'uninstall' ] && server_install_set $build_projects || server_uninstall_set $build_projects
    if echo "$daemon_run"|grep -qi 'y'; then
        (`start_task $build_projects &>builds.out` &)
        echo "正在后台运行中......"
    else
        start_task $build_projects
        rm -f builds.log
    fi
}

script_init() {
    emulate bash 2>/dev/null #zsh仿真模式
    echo -e '' | grep -q 'e' && echo_opt_e='' || echo_opt_e='-e' #dash的echo没有-e选项
    PM=`which apt-get || which yum`
    type curl || type wget || $PM -y install curl wget
    type curl && download_tool_cmd='curl -sko' || download_tool_cmd='wget --no-check-certificate -qO'
    rm -f builds.log builds.out
    clear
}

main() {
    script_init
    option
    run_tasks
}

main