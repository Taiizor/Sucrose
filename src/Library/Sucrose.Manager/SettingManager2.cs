using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Net;
using SMCIPAC = Sucrose.Manager.Converter.IPAddressConverter;
using SMHR = Sucrose.Manager.Helper.Reader;
using SMHW = Sucrose.Manager.Helper.Writer;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Manager
{
    public class SettingManager2
    {
        private Settings _settings = new();
        private static object lockObject = new();
        private readonly string _settingsFilePath;
        private readonly string _settingsFileName;
        private DateTime _lastWrite = DateTime.Now;
        private readonly ReaderWriterLockSlim _lock;
        private readonly FileSystemWatcher _settingsFileWatcher;
        private readonly JsonSerializerSettings _serializerSettings;

        public SettingManager2(string settingsFileName, Formatting formatting = Formatting.Indented, TypeNameHandling typeNameHandling = TypeNameHandling.None)
        {
            _settingsFilePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder, settingsFileName);

            _settingsFileWatcher = new(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder));

            Directory.CreateDirectory(Path.GetDirectoryName(_settingsFilePath));

            _settingsFileWatcher.NotifyFilter = NotifyFilters.LastWrite;

            _settingsFileWatcher.Changed += SettingsFile_Changed;

            _serializerSettings = new JsonSerializerSettings
            {
                TypeNameHandling = typeNameHandling,
                Formatting = formatting,
                Converters =
                {
                    //new SMCEC(),
                    new SMCIPAC(),
                    //new StringEnumConverter()
                }
            };

            _lock = new ReaderWriterLockSlim(); //ReaderWriterLock

            ControlFile();

            _settingsFileName = settingsFileName;

            _settingsFileWatcher.EnableRaisingEvents = true;
        }

        public T GetSetting<T>(string key, T back = default)
        {
            if (_settings.Properties.TryGetValue(key, out object value))
            {
                return ConvertToType<T>(value);
            }

            return back;
        }

        public T GetSettingStable<T>(string key, T back = default)
        {
            if (_settings.Properties.TryGetValue(key, out object value))
            {
                return JsonConvert.DeserializeObject<T>(value.ToString());
            }

            return back;
        }

        public T GetSettingAddress<T>(string key, T back = default)
        {
            if (_settings.Properties.TryGetValue(key, out object value))
            {
                return ConvertToType<T>(value);
            }

            return back;
        }

        public void SetSetting<T>(string key, T value)
        {
            _settings.Properties[key] = ConvertToType<T>(value);

            SaveSetting();
        }

        public void SaveSetting()
        {
            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

                    try
                    {
                        try
                        {
                            Mutex.WaitOne();
                        }
                        catch
                        {
                            //
                        }

                        SMHW.Write(_settingsFilePath, JsonConvert.SerializeObject(_settings, _serializerSettings));
                        _lastWrite = DateTime.Now;
                    }
                    finally
                    {
                        Mutex.ReleaseMutex();
                    }
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }

        public string ReadSetting()
        {
            _lock.EnterReadLock();

            try
            {
                lock (lockObject)
                {
                    using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

                    try
                    {
                        try
                        {
                            Mutex.WaitOne();
                        }
                        catch
                        {
                            //
                        }

                        return SMHR.Read(_settingsFilePath).Result;
                    }
                    finally
                    {
                        Mutex.ReleaseMutex();
                    }
                }
            }
            finally
            {
                _lock.ExitReadLock();
            }
        }

        public void ApplySetting()
        {
            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

                    try
                    {
                        try
                        {
                            Mutex.WaitOne();
                        }
                        catch
                        {
                            //
                        }

                        _settings = new();

                        SMHW.Write(_settingsFilePath, JsonConvert.SerializeObject(_settings, _serializerSettings));
                        _lastWrite = DateTime.Now;
                    }
                    finally
                    {
                        Mutex.ReleaseMutex();
                    }
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }

        public bool CheckFile()
        {
            return File.Exists(_settingsFilePath);
        }

        public string SettingFile()
        {
            return _settingsFilePath;
        }

        private void ControlFile()
        {
            if (CheckFile())
            {
                string json = ReadSetting();

                if (string.IsNullOrEmpty(json))
                {
                    ApplySetting();
                }
                else
                {
                    try
                    {
                        _settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                        if (_settings != null && _settings.Properties != null)
                        {
                            return;
                        }
                        else
                        {
                            ApplySetting();
                        }
                    }
                    catch
                    {
                        ApplySetting();
                    }
                }
            }
            else
            {
                ApplySetting();
            }
        }

        private T ConvertToType<T>(object value)
        {
            Type type = typeof(T);

            if (type == typeof(IPAddress))
            {
                return (T)(object)IPAddress.Parse(value.ToString());
            }
            else if (type == typeof(Uri))
            {
                return (T)(object)new Uri(value.ToString());
            }
            else if (type.IsEnum)
            {
                return (T)Enum.Parse(type, value.ToString());
            }
            else if (type == typeof(KeyValuePair<string, string>))
            {
                string[] parts = value.ToString().Split(':');

                return (T)(object)new KeyValuePair<string, string>(parts[0].Trim(), parts[1].Trim());
            }
            else if (type == typeof(string[]))
            {
                if (value is string[] array)
                {
                    return (T)(object)array;
                }
                else if (value is JArray jArray)
                {
                    return (T)(object)jArray.Select(jValue => (string)jValue).ToArray();
                }
            }
            else if (type == typeof(List<string>))
            {
                if (value is List<string> list)
                {
                    return (T)(object)list;
                }
                else if (value is JArray jArray)
                {
                    return (T)(object)jArray.Select(jValue => (string)jValue).ToList();
                }
            }
            else if (type == typeof(Dictionary<string, string>))
            {
                try
                {
                    return JsonConvert.DeserializeObject<T>(value.ToString());
                }
                catch
                {
                    return JsonConvert.DeserializeObject<T>(JsonConvert.SerializeObject(value));
                }
            }

            return (T)value;
        }

        private void SettingsFile_Changed(object sender, FileSystemEventArgs e)
        {
            if (e.Name == _settingsFileName && File.GetLastWriteTime(_settingsFilePath) > _lastWrite)
            {
                _settings = JsonConvert.DeserializeObject<Settings>(ReadSetting(), _serializerSettings);
                _lastWrite = DateTime.Now;
            }
        }

        private class Settings
        {
            public Dictionary<string, object> Properties { get; set; } = new Dictionary<string, object>();
        }
    }
}