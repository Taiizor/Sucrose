using System.Media;
using System.Windows;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;

namespace Sucrose.Update.View
{
    /// <summary>
    /// LightInfoBox.xaml etkileşim mantığı
    /// </summary>
    public partial class LightInfoBox : Window
    {
        internal LightInfoBox(SSDEUT Type)
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
                default:
                    Sad_Image.Visibility = Visibility.Visible;
                    break;
            }

            SystemSounds.Asterisk.Play();

            Text_Message.Text = SSRER.GetValue("Update", "InfoBox", "TextMessage", $"{Type}");
        }

        private async void Countdown()
        {
            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 5";

            await Task.Delay(1000);

            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 4";

            await Task.Delay(1000);

            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 3";

            await Task.Delay(1000);

            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 2";

            await Task.Delay(1000);

            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 1";

            await Task.Delay(1000);

            Close_Button.Content = $"{SSRER.GetValue("Update", "InfoBox", "CloseText")} 0";

            await Task.Delay(1000);

            Close();
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}