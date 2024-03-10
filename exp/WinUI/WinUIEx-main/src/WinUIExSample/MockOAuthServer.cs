using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace WinUIExSample
{
    /// <summary>
    /// Very basic oauth webserver to mock the OAuth login flow in a browser.
    /// </summary>
    internal class MockOAuthServer : IDisposable
    {
        private CancellationTokenSource cancellationTokenSource = new CancellationTokenSource();

        private HttpListener listener;

        public MockOAuthServer()
        {
            listener = new HttpListener();
            var port = FindAvailablePort();
            Url = $"http://localhost:{port}/";
            listener.Prefixes.Add(Url);
            listener.Start();
            Task.Run(WebServer);
        }

        public string Url { get; }

        private async void WebServer()
        {
            while (!cancellationTokenSource.IsCancellationRequested)
            {
                try
                {
                    var context = await listener.GetContextAsync();

                    if (context.Request.RawUrl == "/oauth/token" && context.Request.HttpMethod == "POST" && context.Request.ContentType == "application/x-www-form-urlencoded")
                    {
                        var headers = context.Request.Headers;
                        Dictionary<string, string> parameters;
                        using (var reader = new StreamReader(context.Request.InputStream))
                        {
                            var body = await reader.ReadToEndAsync();
                            parameters = body.Split('&').Select(x => x.Split('=')).ToDictionary(x => Uri.UnescapeDataString(x[0]), x => Uri.UnescapeDataString(x[1]));
                        }
                        UriBuilder uriBuilder = new UriBuilder(parameters["redirect_uri"]);
                        StringBuilder query = new StringBuilder();
                        if (parameters.ContainsKey("state"))
                            query.Append($"state={Uri.EscapeDataString(parameters["state"])}&");
                        if (parameters["response_type"] == "code")
                            query.Append($"code={Guid.NewGuid()}&");
                        else if (parameters["response_type"] == "token")
                        {
                            query.Append($"access_token={Guid.NewGuid()}&");
                            query.Append($"expires_in=86400&");
                            query.Append($"refresh_token={Guid.NewGuid()}&");
                            query.Append($"refresh_token_expires_in=3600&");
                            query.Append($"token_type=bearer&");
                            if (parameters.ContainsKey("scope"))
                                query.Append($"scope={parameters["scope"]}&");
                        }
                        uriBuilder.Query = query.ToString().Replace("&", "&amp;");
                        using (var writer = new StreamWriter(context.Response.OutputStream))
                        {
                            writer.WriteLine($"""
<html><head><meta http-equiv="Refresh" content="0; URL={uriBuilder.ToString()}" /></head>
<body><div style="border-width:1px;border-style: solid;align:center;padding:30px;margin:20px;background-color:#eee;width:300px">
Signed in. You can close this window now.</div></body></html>
""");
                        }
                    }
                    else
                    {
                        using (var writer = new StreamWriter(context.Response.OutputStream))
                        {
                            writer.WriteLine($"""
<html>
  <head>
    <title>WinUIEx Mocked OAuth Sign in</title>
<body>
<div style="border-width:1px;border-style: solid;align:center;padding:30px;margin:20px;background-color:#eee;width:300px">
<h3>Sign in to WinUIEx's Mocked Server</h3>
<form action="{Url}oauth/token" method="POST">Enter username: <input type="text" name="username" value="user1"><br/>
<input type="submit" value="Sign in" style="background-color:cornflowerblue;color:white;padding:10px;margin-top:10px;border-color:white;width:100px;" />
""");
                            foreach (var key in context.Request.QueryString.AllKeys)
                            {
                                writer.WriteLine($"<input type=\"hidden\" name=\"{key}\" value=\"{context.Request.QueryString[key].Replace("\"", "&quot;")}\" />");
                            }
                            writer.WriteLine("</form></div>");
                            writer.WriteLine("<hr/><b>Values sent:</b><br/>");
                                foreach(var key in context.Request.QueryString.AllKeys)
                            {
                                writer.WriteLine($"&nbsp;&nbsp;&nbsp;{key}: = {context.Request.QueryString[key]}<br/>");
                            }
                            writer.WriteLine("</body></html>");
                        }
                    }
                }
                catch { }
            }
        }

        public void Dispose()
        {
            cancellationTokenSource.Cancel();
            listener.Stop();
        }

        private static int FindAvailablePort()
        {
            TcpListener tcpListener = new TcpListener(IPAddress.Loopback, 0);
            tcpListener.Start();
            int port = ((IPEndPoint)tcpListener.LocalEndpoint).Port;
            tcpListener.Stop();
            return port;
        }
    }
}
