<template>
  <div class="widget" data-tauri-drag-region>
    <div
      v-for="(task, index) in urgentTasks"
      :key="task.id"
      class="task-card"
      :style="cardStyle(index)"
    >
      <div class="task-top" :style="dateBarStyle(index)">
        <span>{{ task.due_date }}</span>
        <span>{{ task.priority }}</span>
      </div>
      <div class="task-bottom">
        <span :style="textStyle(index)">{{ task.title }}</span>
        <div class="task-actions">
          <button class="btn-complete" @click="completeTask(task.id)">complete</button>
          <button class="btn-delete" @click="deleteTask(task.id)">delete</button>
        </div>
      </div>
    </div>

    <div class="widget-footer">
      <div class="add-task" @click="showForm = true">
        <span class="add-icon">✛</span>
        <span>Add task</span>
      </div>
      <div class="open-app" @click="openApp">⚙ Open App</div>
      <div class="drag-handle" @mousedown="startDrag">⠿ drag</div>
    </div>

    <div v-if="showForm" class="form">
      <input v-model="newTask.title" placeholder="Task title" />
      <input v-model="newTask.due_date" type="date" />
      <select v-model="newTask.priority">
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
      </select>
      <button @click="addTask">Save</button>
      <button @click="showForm = false">Cancel</button>
    </div>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

export default {
  data() {
    return {
      tasks: [],
      showForm: false,
      newTask: {
        title: '',
        due_date: '',
        priority: 'medium',
        status: 'pending',
        completed: false,
      },
      colors: {
        color1: '#2d7a9a',
        color2: '#e8e8e8',
        text1: '#ffffff',
        text2: '#000000',
        completeBtn: '#4caf50',
        deleteBtn: '#f44336',
      }
    }
  },
  computed: {
    urgentTasks() {
      return [...this.tasks]
        .filter(t => !t.completed)
        .sort((a, b) => new Date(a.due_date) - new Date(b.due_date))
        .slice(0, 4)
    }
  },
  async mounted() {
    await this.loadTasks()
  },
  methods: {
    async loadTasks() {
      this.tasks = await invoke('get_tasks')
    },
    async addTask() {
      const task = {
        ...this.newTask,
        id: Date.now().toString(),
      }
      await invoke('add_task', { task })
      await this.loadTasks()
      this.showForm = false
      this.newTask = { title: '', due_date: '', priority: 'medium', status: 'pending', completed: false }
    },
    async completeTask(id) {
      await invoke('complete_task', { id })
      await this.loadTasks()
    },
    async deleteTask(id) {
      await invoke('delete_task', { id })
      await this.loadTasks()
    },
   async startDrag(e) {
  const win = getCurrentWindow()
  const pos = await win.outerPosition()
  
  const initX = e.screenX
  const initY = e.screenY
  const initWinX = pos.x
  const initWinY = pos.y

  const onMove = async (e) => {
    const newX = initWinX + (e.screenX - initX)
    const newY = initWinY + (e.screenY - initY)
    await win.setPosition(new (await import('@tauri-apps/api/dpi')).PhysicalPosition(newX, newY))
  }

  const onUp = () => {
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }

  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
},
    cardStyle(index) {
      const isEven = index % 2 === 0
      return {
        backgroundColor: isEven ? this.colors.color1 : this.colors.color2,
        borderRadius: '12px',
        marginBottom: '8px',
        padding: '8px',
      }
    },
    dateBarStyle(index) {
      const isEven = index % 2 === 0
      return {
        backgroundColor: isEven ? this.colors.color2 : this.colors.color1,
        color: isEven ? this.colors.text2 : this.colors.text1,
        borderRadius: '6px',
        padding: '4px 8px',
        display: 'flex',
        justifyContent: 'space-between',
        marginBottom: '6px',
      }
    },
    textStyle(index) {
      const isEven = index % 2 === 0
      return {
        color: isEven ? this.colors.text1 : this.colors.text2,
      }
    },
    async openApp() {
      try {
        await invoke('open_main_window')
      } catch (e) {
        console.error(e)
        alert(e)
      }
    },
  }
}
</script>

<style scoped>
.widget {
  width: 380px;
  padding: 8px;
}

.task-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px;
}

.task-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.btn-complete {
  background: #4caf50;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
}

.btn-delete {
  background: #f44336;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
}

.add-task {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #2d7a9a;
  font-size: 14px;
  margin-top: 4px;
}

.add-icon {
  font-size: 24px;
  font-weight: bold;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
  background: white;
  padding: 10px;
  border-radius: 8px;
}

.form input, .form select {
  padding: 6px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.form button {
  padding: 6px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.widget-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

.open-app {
  cursor: pointer;
  color: #2d7a9a;
  font-size: 14px;
}

.drag-handle {
  color: rgba(255,255,255,0.3);
  font-size: 11px;
  text-align: center;
  cursor: grab;
  padding: 2px;
  user-select: none;
}
</style>