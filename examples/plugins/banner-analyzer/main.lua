--[[
  Banner Analyzer Plugin

  Analyzes service banners to identify services, versions, and additional information.
  Supports common services: HTTP, SSH, FTP, SMTP, MySQL, PostgreSQL, Redis, MongoDB
]]

-- Plugin lifecycle functions
function on_load(config)
    prtip.log("info", "Banner Analyzer plugin loaded")
    return true
end

function on_unload()
    prtip.log("info", "Banner Analyzer plugin unloaded")
end

-- Helper: Extract version from string using pattern
local function extract_version(text, pattern)
    local version = string.match(text, pattern)
    return version
end

-- HTTP banner analysis
local function analyze_http(banner)
    local lower = string.lower(banner)

    -- Apache detection
    if string.match(lower, "apache") then
        local version = extract_version(banner, "Apache/([%d%.]+)")
        local os = nil
        if string.match(lower, "ubuntu") then os = "Linux"
        elseif string.match(lower, "debian") then os = "Linux"
        elseif string.match(lower, "centos") then os = "Linux"
        elseif string.match(lower, "win") then os = "Windows"
        end

        return {
            service = "http",
            product = "Apache",
            version = version,
            os_type = os,
            confidence = version and 0.95 or 0.85
        }
    end

    -- Nginx detection
    if string.match(lower, "nginx") then
        local version = extract_version(banner, "nginx/([%d%.]+)")
        return {
            service = "http",
            product = "nginx",
            version = version,
            confidence = version and 0.95 or 0.85
        }
    end

    -- Microsoft IIS detection
    if string.match(lower, "microsoft%-iis") then
        local version = extract_version(banner, "Microsoft%-IIS/([%d%.]+)")
        return {
            service = "http",
            product = "Microsoft IIS",
            version = version,
            os_type = "Windows",
            confidence = 0.95
        }
    end

    -- Generic HTTP
    if string.match(lower, "^http/") then
        return {
            service = "http",
            confidence = 0.7
        }
    end

    return nil
end

-- SSH banner analysis
local function analyze_ssh(banner)
    local lower = string.lower(banner)

    if string.match(lower, "^ssh") then
        -- OpenSSH detection
        if string.match(lower, "openssh") then
            local version = extract_version(banner, "OpenSSH[_%s]([%d%.p]+)")
            local os = nil
            if string.match(lower, "ubuntu") then os = "Linux"
            elseif string.match(lower, "debian") then os = "Linux"
            elseif string.match(lower, "freebsd") then os = "FreeBSD"
            end

            return {
                service = "ssh",
                product = "OpenSSH",
                version = version,
                os_type = os,
                confidence = 0.95
            }
        end

        -- Generic SSH
        local version = extract_version(banner, "SSH%-([%d%.]+)")
        return {
            service = "ssh",
            version = version,
            confidence = 0.8
        }
    end

    return nil
end

-- FTP banner analysis
local function analyze_ftp(banner)
    local lower = string.lower(banner)

    if string.match(lower, "^220") or string.match(lower, "ftp") then
        -- ProFTPD detection
        if string.match(lower, "proftpd") then
            local version = extract_version(banner, "ProFTPD ([%d%.]+)")
            return {
                service = "ftp",
                product = "ProFTPD",
                version = version,
                confidence = 0.95
            }
        end

        -- vsftpd detection
        if string.match(lower, "vsftpd") then
            local version = extract_version(banner, "vsftpd ([%d%.]+)")
            return {
                service = "ftp",
                product = "vsftpd",
                version = version,
                os_type = "Linux",
                confidence = 0.95
            }
        end

        -- Microsoft FTP Service
        if string.match(lower, "microsoft ftp") then
            return {
                service = "ftp",
                product = "Microsoft FTP Service",
                os_type = "Windows",
                confidence = 0.9
            }
        end

        -- Generic FTP
        return {
            service = "ftp",
            confidence = 0.75
        }
    end

    return nil
end

-- SMTP banner analysis
local function analyze_smtp(banner)
    local lower = string.lower(banner)

    if string.match(lower, "^220") and string.match(lower, "smtp") then
        -- Postfix detection
        if string.match(lower, "postfix") then
            return {
                service = "smtp",
                product = "Postfix",
                confidence = 0.9
            }
        end

        -- Sendmail detection
        if string.match(lower, "sendmail") then
            local version = extract_version(banner, "Sendmail ([%d%.]+)")
            return {
                service = "smtp",
                product = "Sendmail",
                version = version,
                confidence = 0.9
            }
        end

        -- Microsoft Exchange
        if string.match(lower, "microsoft esmtp") then
            return {
                service = "smtp",
                product = "Microsoft Exchange",
                os_type = "Windows",
                confidence = 0.9
            }
        end

        -- Generic SMTP
        return {
            service = "smtp",
            confidence = 0.7
        }
    end

    return nil
end

-- MySQL banner analysis
local function analyze_mysql(banner)
    local lower = string.lower(banner)

    if string.match(lower, "mysql") or string.match(banner, "[\x00-\x09]mysql") then
        local version = extract_version(banner, "([%d%.]+)%-")
        return {
            service = "mysql",
            product = "MySQL",
            version = version,
            confidence = 0.9
        }
    end

    return nil
end

-- PostgreSQL banner analysis
local function analyze_postgresql(banner)
    local lower = string.lower(banner)

    if string.match(lower, "postgresql") then
        local version = extract_version(banner, "PostgreSQL ([%d%.]+)")
        return {
            service = "postgresql",
            product = "PostgreSQL",
            version = version,
            confidence = 0.95
        }
    end

    return nil
end

-- Redis banner analysis
local function analyze_redis(banner)
    local lower = string.lower(banner)

    if string.match(lower, "redis_version") then
        local version = extract_version(banner, "redis_version:([%d%.]+)")
        return {
            service = "redis",
            product = "Redis",
            version = version,
            confidence = 0.95
        }
    end

    return nil
end

-- MongoDB banner analysis
local function analyze_mongodb(banner)
    if string.match(banner, "MongoDB") then
        local version = extract_version(banner, "version ([%d%.]+)")
        return {
            service = "mongodb",
            product = "MongoDB",
            version = version,
            confidence = 0.95
        }
    end

    return nil
end

-- Main analyze_banner function (called by ProRT-IP)
function analyze_banner(banner)
    if not banner or banner == "" then
        return nil
    end

    -- Try each analyzer in sequence
    local result = analyze_http(banner)
        or analyze_ssh(banner)
        or analyze_ftp(banner)
        or analyze_smtp(banner)
        or analyze_mysql(banner)
        or analyze_postgresql(banner)
        or analyze_redis(banner)
        or analyze_mongodb(banner)

    if result then
        prtip.log("info", string.format("Detected %s service (confidence: %.2f)",
            result.service, result.confidence))
    end

    return result
end

-- Active probing function (called by ProRT-IP)
function probe_service(target)
    -- This plugin focuses on passive banner analysis
    -- Active probing could be implemented here for services that don't send banners
    prtip.log("debug", string.format("Probe service called for %s (passive analysis only)",
        target.ip))
    return nil
end
