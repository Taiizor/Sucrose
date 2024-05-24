using System.ComponentModel;
using SSSID = Sucrose.Shared.Store.Interface.Data;

namespace Sucrose.Portal.Services
{
    internal class StoreService
    {
        private ObservableDictionary<string, SSSID> _Info = new();

        public event PropertyChangedEventHandler InfoChanged
        {
            add => _Info.PropertyChanged += value;
            remove => _Info.PropertyChanged -= value;
        }

        public ObservableDictionary<string, SSSID> Info
        {
            get => _Info;
            set
            {
                if (_Info != value)
                {
                    _Info = value;
                    _Info.OnDictionaryChanged();
                }
            }
        }

        public void ProgressPercentage(string Key, double ProgressPercentage)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].ProgressPercentage != ProgressPercentage)
                {
                    _Info[Key].ProgressPercentage = ProgressPercentage;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }

        public void DownloadedFileCount(string Key, int DownloadedFileCount)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].DownloadedFileCount != DownloadedFileCount)
                {
                    _Info[Key].DownloadedFileCount = DownloadedFileCount;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }

        public void TotalFileCount(string Key, int TotalFileCount)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].TotalFileCount != TotalFileCount)
                {
                    _Info[Key].TotalFileCount = TotalFileCount;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }

        public void Percentage(string Key, string Percentage)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].Percentage != Percentage)
                {
                    _Info[Key].Percentage = Percentage;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }

        public void State(string Key, string State)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].State != State)
                {
                    _Info[Key].State = State;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }

        public void Guid(string Key, string Guid)
        {
            if (Info.ContainsKey(Key))
            {
                if (_Info[Key].Guid != Guid)
                {
                    _Info[Key].Guid = Guid;
                    _Info.OnPropertyChanged(Key);
                }
            }
        }
    }

    internal class ObservableDictionary<TKey, TValue> : Dictionary<TKey, TValue>, INotifyPropertyChanged
    {
        public event PropertyChangedEventHandler PropertyChanged;

        public virtual void OnPropertyChanged(string PropertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(PropertyName));
        }

        public new TValue this[TKey Key]
        {
            get => base[Key];
            set
            {
                base[Key] = value;
                OnDictionaryChanged();
                OnPropertyChanged($"{Key}");
            }
        }

        public virtual void OnDictionaryChanged()
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(Count)));
        }
    }
}