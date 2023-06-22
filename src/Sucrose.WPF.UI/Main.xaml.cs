using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.Grpc.Client.Services;
using System.Windows;
using System.Windows.Controls;
using Sucrose.Manager;
using System.Net;

namespace Sucrose.WPF.UI
{
    /// <summary>
    /// Main.xaml etkileşim mantığı
    /// </summary>
    public partial class Main : Window
    {
        SettingsManager SettingsManager = new("Server.json");

        private readonly string Uri1 = "https://www.vegalya.com/3/0wj1biqk.f41/fluid.html";
        private readonly string Uri2 = "https://www.vegalya.com/3/iqdvd4pt.jyo/triangle.html";
        private readonly string Uri3 = "https://www.vegalya.com/3/lgz4xpht.sjn/index.html";
        private readonly string Uri4 = "https://www.vegalya.com/3/mrn3a0m5.0lk/index.html";
        private readonly string Uri5 = "https://www.vegalya.com/3/rtnm43pj.wyl/index.html";
        private readonly string Uri6 = "https://www.vegalya.com/3/xn0quq52.bq2/index.html";
        private readonly string Uri7 = "https://www.vegalya.com/3/wxqfmbno.3vk/index.html";
        private readonly string Uri8 = "https://www.vegalya.com/3/ptpcxvcd.hlz/index.html";
        private readonly string Uri9 = "https://www.vegalya.com/3/nps35xrp.5eh/jellyfish.html";

        public Main()
        {
            InitializeComponent();
        }

        private void Button_Click(object sender, RoutedEventArgs e)
        {
            GeneralServerService.ChannelCreate(SettingsManager.GetSetting<string>("Host"), SettingsManager.GetSettingStable<int>("Port"));
            Websiter.WebsiterClient client = new(GeneralServerService.ChannelInstance);
            //WebsiterChangeResponse response = WebsiterClientService.ChangeAddress(client, "https://www.vegalya.com", true);

            switch ((sender as Button).Content)
            {
                case "Design 1":
                    WebsiterClientService.ChangeAddress(client, Uri1, true);
                    break;
                case "Design 2":
                    WebsiterClientService.ChangeAddress(client, Uri2, true);
                    break;
                case "Design 3":
                    WebsiterClientService.ChangeAddress(client, Uri3, false);
                    break;
                case "Design 4":
                    WebsiterClientService.ChangeAddress(client, Uri4, false);
                    break;
                case "Design 5":
                    WebsiterClientService.ChangeAddress(client, Uri5, false);
                    break;
                case "Design 6":
                    WebsiterClientService.ChangeAddress(client, Uri6, false);
                    break;
                case "Design 7":
                    WebsiterClientService.ChangeAddress(client, Uri7, false);
                    break;
                case "Design 8":
                    WebsiterClientService.ChangeAddress(client, Uri8, false);
                    break;
                case "Design 9":
                    WebsiterClientService.ChangeAddress(client, Uri9, false);
                    break;
                default:
                    break;
            }
        }
    }
}