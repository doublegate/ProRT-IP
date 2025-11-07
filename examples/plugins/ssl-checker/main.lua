--[[
  SSL/TLS Certificate Checker Plugin

  Analyzes SSL/TLS services and certificates to identify:
  - HTTPS servers (port 443, 8443)
  - SMTPS (port 465, 587)
  - IMAPS (port 993)
  - POP3S (port 995)

  Note: This is a demonstration plugin showing integration patterns.
  Full certificate analysis would require ProRT-IP TLS API extensions.
]]

-- Plugin lifecycle functions
function on_load(config)
    prtip.log("info", "SSL Checker plugin loaded")
    return true
end

function on_unload()
    prtip.log("info", "SSL Checker plugin unloaded")
end

-- Common SSL/TLS ports and their services
local ssl_ports = {
    [443] = "https",
    [8443] = "https-alt",
    [465] = "smtps",
    [587] = "submission", -- with STARTTLS
    [993] = "imaps",
    [995] = "pop3s",
    [636] = "ldaps",
    [989] = "ftps-data",
    [990] = "ftps",
    [992] = "telnets",
    [994] = "ircs",
    [3389] = "rdp",  -- RDP over TLS
    [5223] = "xmpps", -- XMPP over SSL
    [5671] = "amqps", -- AMQP over TLS
    [6514] = "syslog-tls",
    [8000] = "https-alt",
    [8080] = "http-proxy", -- may support TLS
    [8443] = "https-alt"
}

-- Analyze banner to detect SSL/TLS hints
function analyze_banner(banner)
    if not banner or banner == "" then
        return nil
    end

    local lower = string.lower(banner)

    -- Check for TLS/SSL error messages (connection without proper TLS handshake)
    if string.match(lower, "tls") or string.match(lower, "ssl")
       or string.match(banner, "\x15\x03") -- TLS alert
       or string.match(banner, "\x16\x03") -- TLS handshake
    then
        prtip.log("info", "Detected TLS/SSL protocol in banner")
        return {
            service = "ssl",
            info = "TLS/SSL encrypted service",
            confidence = 0.7
        }
    end

    -- Check for HTTPS-specific patterns
    if string.match(lower, "https") then
        return {
            service = "https",
            info = "HTTPS service detected from banner",
            confidence = 0.8
        }
    end

    return nil
end

-- Active probing to detect SSL/TLS services
function probe_service(target)
    if not target or not target.ip then
        prtip.log("warn", "Invalid target provided to SSL checker")
        return nil
    end

    prtip.log("debug", string.format("SSL check for %s", target.ip))

    -- Note: This is a demonstration of the plugin architecture
    -- In a full implementation, this would:
    -- 1. Attempt TLS handshake via prtip.connect() with TLS support
    -- 2. Extract certificate information
    -- 3. Analyze certificate validity, expiration, issuer
    -- 4. Return detailed ServiceInfo with certificate data

    -- For now, we demonstrate the service identification pattern
    -- based on common SSL/TLS port conventions

    -- Example: If ProRT-IP exposed port information in target table,
    -- we could do something like:
    --
    -- local service = ssl_ports[target.port]
    -- if service then
    --     return {
    --         service = service,
    --         product = "SSL/TLS Service",
    --         info = "Detected on standard SSL port",
    --         confidence = 0.75
    --     }
    -- end

    -- Placeholder return showing the expected structure
    prtip.log("info", string.format("Would perform SSL analysis for %s", target.ip))

    return nil
end

--[[
  Future Enhancement Ideas (when ProRT-IP TLS API is extended):

  1. Certificate Chain Validation
     - Verify certificate chain completeness
     - Check intermediate certificates
     - Validate root CA trust

  2. Certificate Details Analysis
     - Subject and Issuer DN parsing
     - Subject Alternative Names (SAN) extraction
     - Key usage and extended key usage flags
     - Certificate policies

  3. Security Assessment
     - Certificate expiration checking
     - Weak key detection (< 2048-bit RSA)
     - Self-signed certificate detection
     - Certificate transparency log checking

  4. TLS Protocol Analysis
     - Supported TLS versions (TLS 1.0, 1.1, 1.2, 1.3)
     - Cipher suite enumeration
     - Perfect Forward Secrecy support
     - Weak cipher detection

  5. Vulnerability Checks
     - Heartbleed detection
     - POODLE vulnerability
     - FREAK attack detection
     - LOGJAM vulnerability

  Example Future API Usage:

  function probe_service(target)
      -- Connect with TLS
      local socket_id = prtip.connect_tls(target.ip, 443, 5.0)
      if not socket_id then
          return nil
      end

      -- Get certificate info (hypothetical API)
      local cert_info = prtip.get_certificate(socket_id)
      prtip.close(socket_id)

      if cert_info then
          return {
              service = "https",
              product = cert_info.subject_cn,
              version = cert_info.tls_version,
              info = string.format("Expires: %s", cert_info.not_after),
              confidence = 0.95
          }
      end

      return nil
  end
]]

-- Helper: Check if service is likely encrypted
local function is_ssl_port(port)
    return ssl_ports[port] ~= nil
end

-- Helper: Get service name for SSL port
local function get_ssl_service(port)
    return ssl_ports[port] or "unknown-ssl"
end
