using System;
using System.Collections;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using Microsoft.UI.Xaml;
using System.Runtime.InteropServices;
using WinUIEx;

namespace WinUIExSample
{
    /// <summary>
    /// Provides application-specific behavior to supplement the default Application class.
    /// </summary>
    public partial class App : Application
    {
        /// <summary>
        /// Initializes the singleton application object.  This is the first line of authored code
        /// executed, and as such is the logical equivalent of main() or WinMain().
        /// </summary>
        public App()
        {
            if (WebAuthenticator.CheckOAuthRedirectionActivation())
                return;
            this.InitializeComponent();
            int length = 0;
            var sb = new System.Text.StringBuilder(0);
            int result = GetCurrentPackageFullName(ref length, sb);
            if(result == 15700L)
            {
                // Not a packaged app. Configure file-based persistence instead
                WinUIEx.WindowManager.PersistenceStorage = new FilePersistence("WinUIExPersistence.json");
            }
        }
        
        /// <summary>
        /// Invoked when the application is launched normally by the end user.  Other entry points
        /// will be used such as when the application is launched to open a specific file.
        /// </summary>
        /// <param name="args">Details about the launch request and process.</param>
        protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
        {
            // m_window = new MainWindow();
            // m_window.Activate();
            var splash = new SplashScreen(typeof(MainWindow));
            splash.Completed += (s, e) => m_window = (WindowEx)e;
        }

        private WindowEx m_window;

        public WindowEx MainWindow => m_window;

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern int GetCurrentPackageFullName(ref int packageFullNameLength, System.Text.StringBuilder packageFullName);

        private class FilePersistence : IDictionary<string, object>
        {
            private readonly Dictionary<string, object> _data = new Dictionary<string, object>();
            private readonly string _file;

            public FilePersistence(string filename)
            {
                _file = filename;
                try
                {
                    if (File.Exists(filename))
                    {
                        var jo = System.Text.Json.Nodes.JsonObject.Parse(File.ReadAllText(filename)) as JsonObject;
                        foreach(var node in jo)
                        {
                            if (node.Value is JsonValue jvalue && jvalue.TryGetValue<string>(out string value))
                                _data[node.Key] = value;
                        }
                    }
                }
                catch { }
            }
            private void Save()
            {
                JsonObject jo = new JsonObject();
                foreach(var item in _data)
                {
                    if (item.Value is string s) // In this case we only need string support. TODO: Support other types
                        jo.Add(item.Key, s);
                }
                File.WriteAllText(_file, jo.ToJsonString());
            }
            public object this[string key] { get => _data[key]; set { _data[key] = value; Save();} }

            public ICollection<string> Keys => _data.Keys;

            public ICollection<object> Values => _data.Values;

            public int Count => _data.Count;

            public bool IsReadOnly => false;

            public void Add(string key, object value)
            {
                _data.Add(key, value); Save();
            }

            public void Add(KeyValuePair<string, object> item)
            {
                _data.Add(item.Key, item.Value); Save();
            }

            public void Clear()
            {
                _data.Clear(); Save();
            }

            public bool Contains(KeyValuePair<string, object> item) => _data.Contains(item);

            public bool ContainsKey(string key) => _data.ContainsKey(key);

            public void CopyTo(KeyValuePair<string, object>[] array, int arrayIndex) => throw new NotImplementedException(); // TODO

            public IEnumerator<KeyValuePair<string, object>> GetEnumerator() => throw new NotImplementedException(); // TODO

            public bool Remove(string key) => throw new NotImplementedException(); // TODO

            public bool Remove(KeyValuePair<string, object> item) => throw new NotImplementedException(); // TODO

            public bool TryGetValue(string key, [MaybeNullWhen(false)] out object value) => throw new NotImplementedException(); // TODO

            IEnumerator IEnumerable.GetEnumerator() => throw new NotImplementedException(); // TODO
        }
    }
}
