-- file: lua/backend-baidu.lua

-- 引入所需的模块
local http = require 'http'
local backend = require 'backend'

-- 定义局部变量用于字符串操作
local char = string.char
local byte = string.byte
local find = string.find
local sub = string.sub

-- 从backend模块中获取常量
local ADDRESS = backend.ADDRESS
local PROXY = backend.PROXY
local DIRECT_WRITE = backend.SUPPORT.DIRECT_WRITE

local SUCCESS = backend.RESULT.SUCCESS
local HANDSHAKE = backend.RESULT.HANDSHAKE
local DIRECT = backend.RESULT.DIRECT

-- 从backend模块中获取函数
local ctx_uuid = backend.get_uuid
local ctx_proxy_type = backend.get_proxy_type
local ctx_address_type = backend.get_address_type
local ctx_address_host = backend.get_address_host
local ctx_address_bytes = backend.get_address_bytes
local ctx_address_port = backend.get_address_port
local ctx_write = backend.write
local ctx_free = backend.free
local ctx_debug = backend.debug

-- 定义一个用于存储标志的表
local flags = {}
local kHttpHeaderSent = 1
local kHttpHeaderReceived = 2  -- 修正拼写错误

-- 回调函数：返回直接写入标志
function wa_lua_on_flags_cb(ctx)
    return DIRECT_WRITE
end

-- 回调函数：处理握手逻辑
function wa_lua_on_handshake_cb(ctx)
    local uuid = ctx_uuid(ctx)  -- 获取上下文的UUID

    if flags[uuid] == kHttpHeaderReceived then  -- 如果标志为已接收HTTP头，返回true
        return true
    end

    if flags[uuid] ~= kHttpHeaderSent then  -- 如果标志不为已发送HTTP头
        local host = ctx_address_host(ctx)  -- 获取目标主机
        local port = ctx_address_port(ctx)  -- 获取目标端口
        local res = 'CONNECT ' .. host .. ':' .. port .. ' HTTP/1.1\r\n' ..  -- 构建HTTP CONNECT请求
                    'Host: cloudnproxy.baidu.com:443\r\n' ..
                    'User-Agent: Mozilla/5.0 (Linux; Android 12; RMX3300 Build/SKQ1.211019.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/97.0.4692.98 Mobile Safari/537.36 T7/13.32 SP-engine/2.70.0 baiduboxapp/13.32.0.10 (Baidu; P1 12) NABar/1.0\r\n'..
                    'Proxy-Connection: Keep-Alive\r\n'..
                    'X-T5-Auth: 2924846542\r\n\r\n'
        ctx_write(ctx, res)  -- 发送HTTP CONNECT请求
        flags[uuid] = kHttpHeaderSent  -- 更新标志为已发送HTTP头
    end

    return false  -- 返回false表示握手未完成
end

-- 回调函数：处理读取数据逻辑
function wa_lua_on_read_cb(ctx, buf)
    ctx_debug('wa_lua_on_read_cb')  -- 调试信息
    local uuid = ctx_uuid(ctx)  -- 获取上下文的UUID
    if flags[uuid] == kHttpHeaderSent then  -- 如果标志为已发送HTTP头
        flags[uuid] = kHttpHeaderReceived  -- 更新标志为已接收HTTP头
        return HANDSHAKE, nil  -- 返回握手状态
    end
    return DIRECT, buf  -- 返回直接传输数据
end

-- 回调函数：处理写入数据逻辑
function wa_lua_on_write_cb(ctx, buf)
    ctx_debug('wa_lua_on_write_cb')  -- 调试信息
    return DIRECT, buf  -- 返回直接传输数据
end

-- 回调函数：处理关闭连接逻辑
function wa_lua_on_close_cb(ctx)
    ctx_debug('wa_lua_on_close_cb')  -- 调试信息
    local uuid = ctx_uuid(ctx)  -- 获取上下文的UUID
    flags[uuid] = nil  -- 清除标志
    ctx_free(ctx)  -- 释放上下文资源
    return SUCCESS  -- 返回成功状态
end
