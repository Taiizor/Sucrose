using System.ComponentModel;
using SSSID = Sucrose.Shared.Store.Interface.Data;

namespace Sucrose.Portal.Services
{
    internal class StoreService
    {
        public event PropertyChangedEventHandler InfoChanged
        {
            add => Info.PropertyChanged += value;
            remove => Info.PropertyChanged -= value;
        }

        public ObservableDictionary<string, SSSID> Info
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    field.OnDictionaryChanged();
                }
            }
        } = new();

        public void ProgressPercentage(string Key, double ProgressPercentage)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].ProgressPercentage != ProgressPercentage)
                {
                    Info[Key].ProgressPercentage = ProgressPercentage;
                    Info.OnPropertyChanged(Key);
                }
            }
        }

        public void DownloadedFileCount(string Key, int DownloadedFileCount)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].DownloadedFileCount != DownloadedFileCount)
                {
                    Info[Key].DownloadedFileCount = DownloadedFileCount;
                    Info.OnPropertyChanged(Key);
                }
            }
        }

        public void TotalFileCount(string Key, int TotalFileCount)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].TotalFileCount != TotalFileCount)
                {
                    Info[Key].TotalFileCount = TotalFileCount;
                    Info.OnPropertyChanged(Key);
                }
            }
        }

        public void Percentage(string Key, string Percentage)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].Percentage != Percentage)
                {
                    Info[Key].Percentage = Percentage;
                    Info.OnPropertyChanged(Key);
                }
            }
        }

        public void State(string Key, string State)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].State != State)
                {
                    Info[Key].State = State;
                    Info.OnPropertyChanged(Key);
                }
            }
        }

        public void Guid(string Key, string Guid)
        {
            if (Info.ContainsKey(Key))
            {
                if (Info[Key].Guid != Guid)
                {
                    Info[Key].Guid = Guid;
                    Info.OnPropertyChanged(Key);
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