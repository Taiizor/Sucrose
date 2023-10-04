using System.Media;
using System.Windows;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Update.View
{
    /// <summary>
    /// DarkInfoBox.xaml etkileşim mantığı
    /// </summary>
    public partial class DarkInfoBox : Window
    {
        internal DarkInfoBox(SSDEUT Type)
        {
            InitializeComponent();

            switch (Type)
            {
                case SSDEUT.Empty:
                    Info_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Update:
                    Firework_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Status:
                    Warn_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Extract:
                    Bang_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Network:
                    Error_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Updating:
                    Countdown();
                    Confetti_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Releases:
                    Warn_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Download:
                    Defective_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Condition:
                    Winter_Image.Visibility = Visibility.Visible;
                    break;
                default:
                    Sad_Image.Visibility = Visibility.Visible;
                    break;
            }

            SystemSounds.Asterisk.Play();

            SourceInitialized += DarkInfoBox_SourceInitialized;

            Text_Message.Text = SSRER.GetValue("Update", "InfoBox", "TextMessage", $"{Type}");
        }

        private async void Countdown()
        {
            for (int Count = 5; Count >= 0; Count--)
            {
                Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} {Count}";

                await Task.Delay(1000);
            }

            Close();
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void DarkInfoBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }
    }
}