using System.Windows;
using System.Windows.Controls;
using SGCSWCS = Sucrose.Grpc.Client.Services.WebsiterClientService;
using SGCWWC = Sucrose.Grpc.Common.Websiter.WebsiterClient;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Portal
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private readonly string Uri1 = "https://www.vegalya.com/3/0wj1biqk.f41/fluid.html";
        private readonly string Uri2 = "https://www.vegalya.com/3/0wj1biqk.f412/fluid.html";
        private readonly string Uri3 = "https://www.vegalya.com/3/iqdvd4pt.jyo/triangle.html";
        private readonly string Uri4 = "https://www.vegalya.com/3/lgz4xpht.sjn/index.html";
        private readonly string Uri5 = "https://www.vegalya.com/3/mrn3a0m5.0lk/index.html";
        private readonly string Uri6 = "https://www.vegalya.com/3/rtnm43pj.wyl/index.html";
        private readonly string Uri7 = "https://www.vegalya.com/3/xn0quq52.bq2/index.html";
        private readonly string Uri8 = "https://www.vegalya.com/3/wxqfmbno.3vk/index.html";
        private readonly string Uri9 = "https://www.vegalya.com/3/ptpcxvcd.hlz/index.html";
        private readonly string Uri10 = "https://www.vegalya.com/3/nps35xrp.5eh/jellyfish.html";

        public Main()
        {
            InitializeComponent();
        }

        private void Button_Click(object sender, RoutedEventArgs e)
        {
            SGSGSS.ChannelCreate(SMMI.ServerManager.GetSetting<string>("Host"), SMMI.ServerManager.GetSettingStable<int>("Port"));
            SGCWWC Client = new(SGSGSS.ChannelInstance);

            switch ((sender as Button).Content)
            {
                case "Design 1":
                    SGCSWCS.ChangeAddress(Client, Uri1, true);
                    break;
                case "Design 2":
                    SGCSWCS.ChangeAddress(Client, Uri2, true);
                    break;
                case "Design 3":
                    SGCSWCS.ChangeAddress(Client, Uri3, true);
                    break;
                case "Design 4":
                    SGCSWCS.ChangeAddress(Client, Uri4, false);
                    break;
                case "Design 5":
                    SGCSWCS.ChangeAddress(Client, Uri5, true);
                    break;
                case "Design 6":
                    SGCSWCS.ChangeAddress(Client, Uri6, true);
                    break;
                case "Design 7":
                    SGCSWCS.ChangeAddress(Client, Uri7, false);
                    break;
                case "Design 8":
                    SGCSWCS.ChangeAddress(Client, Uri8, true);
                    break;
                case "Design 9":
                    SGCSWCS.ChangeAddress(Client, Uri9, true);
                    break;
                case "Design 10":
                    SGCSWCS.ChangeAddress(Client, Uri10, true);
                    break;
                default:
                    break;
            }
        }
    }
}