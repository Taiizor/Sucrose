using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Media;
using Google.Protobuf.WellKnownTypes;
using Sucrose.Portal.Views.Controls;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// GeneralSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class GeneralSettingPage : Page
    {
        public GeneralSettingPage()
        {
            InitializeComponent();
        }

        private async Task Start()
        {
            //bir ayarlar dizisi oluştur ve ona uygun usercontrolü (controls) FrameSetting'e Children
            //olarak ekle

            TextBlock Tb1 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Görünüş & Davranış"
            };
            TextBlock Tb2 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Uygulama"
            };
            TextBlock Tb3 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Sistem"
            };

            Expander Ex1 = new()
            {
                Header = "This text is in the header",
                Content = "This is in the content",
                Margin = new Thickness(0, 10, 0, 0)
            };
            Expander Ex2 = new()
            {
                Header = "This text is in the header",
                Content = "This is in the content",
                Margin = new Thickness(0, 10, 0, 0)
            };
            Expander Ex3 = new()
            {
                Header = "This text is in the header",
                Content = "This is in the content",
                Margin = new Thickness(0, 10, 0, 0)
            };
            Expander Ex4 = new()
            {
                Header = "This text is in the header",
                Content = "This is in the content",
                Margin = new Thickness(0, 10, 0, 0)
            };
            Expander Ex5 = new()
            {
                Header = "This text is in the header",
                Content = "This is in the content",
                Margin = new Thickness(0, 10, 0, 0)
            };

            FrameSetting.Children.Add(Tb1);

            FrameSetting.Children.Add(Ex1);
            FrameSetting.Children.Add(Ex2);
            FrameSetting.Children.Add(Ex3);

            FrameSetting.Children.Add(Tb2);

            FrameSetting.Children.Add(Ex4);
            FrameSetting.Children.Add(Ex5);

            FrameSetting.Children.Add(Tb3);


            ExpanderCard CustomExpander = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            CustomExpander.HeaderFrame = new Slider()
            {
                TickPlacement = TickPlacement.Both,
                IsSnapToTickEnabled = false,
                TickFrequency = 20,
                Width = 200,
                Value = 50
            };

            ComboBox CB = new();

            CB.Items.Add("Test1");
            CB.Items.Add("Test2");
            CB.Items.Add("Test3");
            CB.Items.Add("Test4");
            CB.Items.Add("Test5");
            CB.Items.Add("Test6");

            CB.SelectedIndex = 0;

            CustomExpander.FooterCard = CB;

            FrameSetting.Children.Add(CustomExpander);




            foreach (string List in new List<string>())
            {
                //

                await Task.Delay(25);
            }

            await Task.Delay(500);

            FrameSetting.Visibility = Visibility.Visible;
            ProgressSetting.Visibility = Visibility.Collapsed;
        }

        private async void GridSetting_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }
    }
}
