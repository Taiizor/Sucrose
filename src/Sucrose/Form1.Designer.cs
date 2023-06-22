namespace Sucrose
{
    partial class Form1
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.crownLabel1 = new ReaLTaiizor.Controls.CrownLabel();
            this.crownLabel2 = new ReaLTaiizor.Controls.CrownLabel();
            this.crownButton1 = new ReaLTaiizor.Controls.CrownButton();
            this.crownButton2 = new ReaLTaiizor.Controls.CrownButton();
            this.SuspendLayout();
            // 
            // crownLabel1
            // 
            this.crownLabel1.AutoSize = true;
            this.crownLabel1.Font = new System.Drawing.Font("Segoe UI", 25F);
            this.crownLabel1.ForeColor = System.Drawing.Color.FromArgb(((int)(((byte)(220)))), ((int)(((byte)(220)))), ((int)(((byte)(220)))));
            this.crownLabel1.Location = new System.Drawing.Point(12, 9);
            this.crownLabel1.Name = "crownLabel1";
            this.crownLabel1.Size = new System.Drawing.Size(209, 46);
            this.crownLabel1.TabIndex = 2;
            this.crownLabel1.Text = "crownLabel1";
            // 
            // crownLabel2
            // 
            this.crownLabel2.AutoSize = true;
            this.crownLabel2.Font = new System.Drawing.Font("Segoe UI", 25F);
            this.crownLabel2.ForeColor = System.Drawing.Color.FromArgb(((int)(((byte)(220)))), ((int)(((byte)(220)))), ((int)(((byte)(220)))));
            this.crownLabel2.Location = new System.Drawing.Point(12, 55);
            this.crownLabel2.Name = "crownLabel2";
            this.crownLabel2.Size = new System.Drawing.Size(209, 46);
            this.crownLabel2.TabIndex = 3;
            this.crownLabel2.Text = "crownLabel2";
            // 
            // crownButton1
            // 
            this.crownButton1.Location = new System.Drawing.Point(130, 104);
            this.crownButton1.Name = "crownButton1";
            this.crownButton1.Padding = new System.Windows.Forms.Padding(5);
            this.crownButton1.Size = new System.Drawing.Size(112, 43);
            this.crownButton1.TabIndex = 0;
            this.crownButton1.TabStop = false;
            this.crownButton1.Text = "Hide Try";
            this.crownButton1.Click += new System.EventHandler(this.CrownButton1_Click);
            // 
            // crownButton2
            // 
            this.crownButton2.Location = new System.Drawing.Point(12, 104);
            this.crownButton2.Name = "crownButton2";
            this.crownButton2.Padding = new System.Windows.Forms.Padding(5);
            this.crownButton2.Size = new System.Drawing.Size(112, 43);
            this.crownButton2.TabIndex = 0;
            this.crownButton2.TabStop = false;
            this.crownButton2.Text = "Show Try";
            this.crownButton2.Click += new System.EventHandler(this.CrownButton2_Click);
            // 
            // Form1
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(96F, 96F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Dpi;
            this.ClientSize = new System.Drawing.Size(686, 283);
            this.Controls.Add(this.crownButton2);
            this.Controls.Add(this.crownButton1);
            this.Controls.Add(this.crownLabel2);
            this.Controls.Add(this.crownLabel1);
            this.MaximizeBox = false;
            this.Name = "Form1";
            this.Text = "Form1";
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private ReaLTaiizor.Controls.CrownLabel crownLabel1;
        private ReaLTaiizor.Controls.CrownLabel crownLabel2;
        private ReaLTaiizor.Controls.CrownButton crownButton1;
        private ReaLTaiizor.Controls.CrownButton crownButton2;
    }
}