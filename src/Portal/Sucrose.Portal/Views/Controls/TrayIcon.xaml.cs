using System.Windows.Input;
using Wpf.Ui.Controls;
using SMMCU = Sucrose.Memory.Manage.Constant.User;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// TrayIcon.xaml etkileşim mantığı
    /// </summary>
    public partial class TrayIcon : ContentDialog, IDisposable
    {
        private bool IsClose { get; set; } = true;

        public TrayIcon() : base(SPMI.ContentDialogService.GetDialogHost())
        {
            InitializeComponent();

            Countdown();
        }

        private async void Countdown()
        {
            for (int Count = 5; Count >= 0; Count--)
            {
                CloseButtonText = $"{SRER.GetValue("Portal", "TrayIcon", "Close")} {Count}";

                await Task.Delay(1000);
            }

            CloseButtonText = SRER.GetValue("Portal", "TrayIcon", "Close");

            IsClose = false;
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (IsClose)
            {
                return;
            }

            base.OnButtonClick(Button);
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key is Key.Enter or Key.Escape)
            {
                if (IsClose)
                {
                    e.Handled = true;
                }
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);

            SMMI.UserSettingManager.SetSetting(SMMCU.HintTrayIcon, false);
        }
    }
}