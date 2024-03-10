using System;
using System.Collections.Generic;
using System.Text.Json.Nodes;

namespace WinUIEx
{
    /// <summary>
    /// Web Authenticator result parsed from the callback Url.
    /// </summary>
    /// <seealso cref="WebAuthenticator"/>
    public class WebAuthenticatorResult
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="WebAuthenticatorResult"/> class.
        /// </summary>
        /// <param name="callbackUrl">Callback url</param>
        public WebAuthenticatorResult(Uri callbackUrl)
        {
            var str = string.Empty;
            if (!string.IsNullOrEmpty(callbackUrl.Fragment))
                str = callbackUrl.Fragment.Substring(1);
            else if (!string.IsNullOrEmpty(callbackUrl.Query))
                str = callbackUrl.Query;
            var query = System.Web.HttpUtility.ParseQueryString(str);
            foreach (string key in query.Keys)
            {
                if (key is null) continue;
                if(key == "state")
                {
                    try
                    {
                        var jsonObject = System.Text.Json.Nodes.JsonObject.Parse(query[key] ?? "{}") as JsonObject;

                        if (jsonObject is not null && jsonObject.ContainsKey("state") && jsonObject["state"] is JsonValue jvalue && jvalue.TryGetValue<string>(out string? value))
                        {
                            Properties[key] = value;
                        }
                        continue;
                    }
                    catch { }
                }
                Properties[key] = query[key] ?? String.Empty;
            }
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="WebAuthenticatorResult"/> class.
        /// </summary>
        /// <param name="values">Values from the authentication callback url</param>
        public WebAuthenticatorResult(Dictionary<string, string> values)
        {
            foreach (var value in values)
                Properties[value.Key] = value.Value;
        }

        /// <summary>
        /// The dictionary of key/value pairs parsed form the callback URI's querystring.
        /// </summary>
        public Dictionary<string, string> Properties { get; } = new Dictionary<string, string>();

        /// <summary>
        /// Gets the value for the <c>access_token</c> key.
        /// </summary>
        /// <value>Access Token parsed from the callback URI <c>access_token</c> parameter.</value>
        public string AccessToken => GetValue("access_token");

        /// <summary>
        /// Gets the value for the <c>refresh_token</c> key.
        /// </summary>
        /// <value>Refresh Token parsed from the callback URI <c>refresh_token</c> parameter.</value>
        public string RefreshToken => GetValue("refresh_token");

        /// <summary>
        /// Gets the value for the <c>id_token</c> key.
        /// </summary>
        public string IdToken => GetValue("id_token");

        /// <summary>
        /// Gets the expiry date as calculated by the timestamp of when the result was created plus the value in seconds for the <c>expires_in</c> key.
        /// </summary>
        /// <value>Timestamp of the creation of the object instance plus the <c>expires_in</c> seconds parsed from the callback URI.</value>
        public DateTimeOffset? RefreshTokenExpiresIn
        {
            get
            {
                if (Properties.TryGetValue("refresh_token_expires_in", out var value))
                {
                    if (int.TryParse(value, out var i))
                        return DateTimeOffset.UtcNow.AddSeconds(i);
                }

                return null;
            }
        }

        /// <summary>
        /// The expiry date as calculated by the timestamp of when the result was created plus the value in seconds for the <c>expires_in</c> key.
        /// </summary>
        /// <value>Timestamp of the creation of the object instance plus the <c>expires_in</c> seconds parsed from the callback URI.</value>
        public DateTimeOffset? ExpiresIn
        {
            get
            {
                if (Properties.TryGetValue("expires_in", out var value))
                {
                    if (int.TryParse(value, out var i))
                        return DateTimeOffset.UtcNow.AddSeconds(i);
                }

                return null;
            }
        }

        private string GetValue(string key)
        {
            if (Properties.TryGetValue(key, out var value))
                return value;
            return string.Empty;
        }
    }
}
